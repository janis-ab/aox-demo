//! Module that implements rudimentary rate limiting capabilities.
//!
//! # Possible future improvements
//! 1. Improve resistance for unsynchronized clocks between systems. Even if
//! current algorithm tries to mitigate unsynced clock impact on rate limiting,
//! it can be further improved.
//! 2. Rewrite this module as standalone crate that implements more rate
//! limiting response headers. I.e. currently this is catered to specific
//! endpoint that returns X-RateLimit-Limit header, but other endpoints use
//! X-Rate-Limit-Limit, some return status 429 "Too many requests", etc. This
//! can be implemented so that common code could be reused with various
//! endpoints.
//! 3. Currently if remote endpoint does not return any rate imiting information,
//! code emmits error for each request. We should handle such case with sane
//! default values instead or emit error once.



use std::time::{
    Duration,
    SystemTime,
    UNIX_EPOCH,
};

use reqwest::{
    Response,
    StatusCode,
};



#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    limit: u64,
    remaining: u64,
    start: SystemTime,
    reset: SystemTime,
}



#[derive(Debug, Clone, Default)]
pub struct RateLimit {
    cur: Option<RateLimitInfo>,
    prev: Option<RateLimitInfo>,
}



impl RateLimit {
    /// Start rate limiting current request.
    ///
    /// This method must be called on each request that must be rate limited.
    pub fn start(&mut self) {
        self.prev = self.cur.clone();
        self.cur = None;
    }



    /// Reset rate limiting information.
    pub fn reset(&mut self) {
        self.cur = None;
        self.prev = None;
    }



    /// Update RateLimit struct information from HTTP response status and
    /// headers.
    ///
    /// If API endpoint returns x-ratelimit-* HTTP headers, RateLimit uses
    /// information from it to update it's internal state. In further calls
    /// to adjust next request this information is taken into consideration.
    ///
    /// If current rate limit is already calculated by HTTP status 429 "Too many
    /// requests", then this method does not update internal state unless HTTP
    /// headers contain more restrictive rate limiting.
    pub fn update_from_response(&mut self, start: &SystemTime, r: &Response) {
        let hm = r.headers();

        // Common macro that extracts u64 or returns None from this method.
        macro_rules! u64_extract {
            ($header_name:expr) => {{
                let Some(val) = hm.get($header_name) else {
                    return
                };

                let Ok(val) = val.to_str() else {
                    return
                };

                let Ok(val) = val.parse::<u64>() else {
                    return
                };

                val
            }}
        }

        if r.status() == StatusCode::TOO_MANY_REQUESTS {
            if let Some(..) = hm.get("retry-after") {
                // TODO: implement such status:429 limit handling.
            }
        }

        //
        // Overwrite rate limiting info from x-ratelimit headers, if that is
        // available.
        //

        let limit = u64_extract!("x-ratelimit-limit");
        let remaining = u64_extract!("x-ratelimit-remaining");
        let reset = u64_extract!("x-ratelimit-reset");

        let reset = UNIX_EPOCH + Duration::from_secs(reset);

        let rl_info = RateLimitInfo {
            limit, remaining,
            start: *start,
            reset,
        };

        // Overwrite current only if headers contain more-strict rate limiting.
        if let Some(cur) = &mut self.cur {
            if cur.reset < rl_info.reset {
                cur.reset = rl_info.reset;
            }

            if cur.remaining > rl_info.remaining {
                cur.remaining = rl_info.remaining;
            }

            if cur.limit > rl_info.limit {
                cur.limit = rl_info.limit;
            }

            return
        }

        self.cur = Some(rl_info);
    }



    /// Adjust time for next request based on rate limit.
    pub fn ts_next_req_adjust(&self, ts_next_req: &mut SystemTime) {
        let Some(ref prev) = self.prev else {
            return self.ts_next_req_adjust_prev_none(ts_next_req)
        };

        // If no rate limiting information is loaded, it is not possible to
        // rate limit requests. This error should not happen unless RateLimiter
        // is not used propperly..
        let Some(ref cur) = self.cur else {
            eprintln!(concat!("ERROR: There is not RateLimit information",
                " loaded, requested rate limiting policy might not be",
                " honored properly."
            ));

            return
        };

        // If request is scheduled after rate limiting window reset, it will not
        // exceed rate limit imposed by API endpoint.
        // This code path should always execute for appropriateley configured
        // system.
        // This works correctly only if system clocks are synchronized well
        // enough.
        if *ts_next_req >= cur.reset {
            return
        }

        // If rate-limit reached, wait till next window.
        if cur.remaining < 1 {
            *ts_next_req = cur.reset;
            return
        }

        // Measure interval between our requests. Use this instead of absolute
        // clock value to minimize impact on un-synced clocks between our system
        // and API endpoint.
        let Ok(request_interval) = cur.start.duration_since(prev.start) else {
            // There is nothing much we can do if rate limit window can not be
            // calculated as to fall back to configured values.
            eprintln!(concat!("WARNING: Clock may have gone backwards,",
                " requested rate limiting policy might not be honored properly."
            ));

            return
        };

        let Ok(win_duration) = cur.reset.duration_since(cur.start) else {
            // API endpoint should never return timestamp that is before request
            // start unless clock on any of involved systems is out of sync. In
            // such an occurance, we ignore dynamic rate limiting calculations
            // and use configured ones.
            eprintln!(concat!("WARNING: Host and enpoint clocks are too far",
                " out of sync, requested rate limiting policy might not be",
                " honored properly. Possible solutions to this:\n",
                "1) Ensure that system clock is correct and stable,\n",
                "2) Increase request period (it is possible that remote",
                " endpoint can not hadnle that may requests),\n",
                "3) Check that in case if remote endpoint is load balanced",
                " between multiple nodes, that clocks between all nodes are",
                " synchronized."
            ));

            return
        };

        let d = win_duration.as_millis();
        let allowed_duration_ms = d / cur.remaining as u128;

        // We can confidently drop unused bits without loosing any meaningful
        // resolution.
        let rl_duration = Duration::from_millis(allowed_duration_ms as u64);

        // If system is making requests on bigger intervals than rate limit
        // requires to do, we can use that duration.
        if request_interval > rl_duration {
            return
        }

        // Since we must have duration of seconds between requests, we must
        // adjust so that next request is number of seconds since the previous.
        *ts_next_req = cur.start + rl_duration;
    }



    // Adjust request timestamp without any information about previous request
    // time and rate limits.
    //
    // This function should be called only for the first request made.
    #[inline]
    fn ts_next_req_adjust_prev_none(&self, ts_next_req: &mut SystemTime) {
        let Some(ref cur) = self.cur else {
            eprintln!(concat!("ERROR: There is not RateLimit information",
                " loaded, requested rate limiting policy might not be",
                " honored properly."
            ));

            return
        };

        if *ts_next_req >= cur.reset {
            return
        }

        // Since this is the first request, and we are allowed to make more
        // requests, there is no need to rate-limit next.
        // We trust that system configuration good enough for service to work
        // properly.
        if cur.remaining > 0 {
            return
        }

        // Being here is not a good start. Because if local system time is out
        // of sync with remote clock, we are using time to sleep based on
        // remote clock. But there is nothing much we can do if system time
        // is not set correctly.

        eprintln!(concat!("WARNING: API endpoint call rate limit was hit on",
            " first request"
        ));

        if cur.reset > *ts_next_req {
            *ts_next_req = cur.reset;
        }
    }
}


