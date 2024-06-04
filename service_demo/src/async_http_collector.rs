use std::{
    sync::{
        Arc,
        atomic::Ordering,
    },
    time::{
        SystemTime,
        Duration,
    }
};

use tokio::{
    time::sleep,
    sync::mpsc,
};

use reqwest::StatusCode;

use serde::Deserialize;

use crate::{
    shared_state::SharedState,
    rate_limit::RateLimit,
    price_info::{
        PriceInfo,
        Symbol,
    },
};



pub struct AsyncHTTPCollector {
    tx: mpsc::Sender<PriceInfo>,
    url: String,
    request_period: u64,
}



impl AsyncHTTPCollector {
    pub fn new(url: &str, tx: mpsc::Sender<PriceInfo>) -> Self {
        Self {
            tx,
            url: url.to_string(),
            request_period: 1000,
        }
    }


    /// Set request period in milliseconds. You should use 500 or more not to
    /// overwhelm endpoint API.
    ///
    /// This describes how long should be the pause between requests to data
    /// endpoint.
    pub fn request_period_millis_set(&mut self, request_period: u64) {
        self.request_period = request_period;
    }
}



/// Structure that stores deserialized response from crypto rates endpoint.
#[derive(Deserialize, Debug, Clone)]
struct DecodedTicker {
    // At the moment we are not using this.
    // symbol: String,

    #[serde(rename(deserialize = "rateUsd"))]
    rate_usd: String,
}



#[derive(Deserialize, Debug, Clone)]
struct DecodedBody {
    data: DecodedTicker,
    timestamp: u64,
}



pub async fn main(collector: AsyncHTTPCollector, shared_state: Arc<SharedState>) {
    // Desired/targeted request period
    let req_period = Duration::from_millis(collector.request_period);

    let mut intr = shared_state.shut_down.load(Ordering::Relaxed);
    let mut rate_limit = RateLimit::default();

    while intr == 0 {
        // Relaxed load, because we do not care on nanosecond shut down
        // precission. We just have to shut down at some point.
        intr = shared_state.shut_down.load(Ordering::Relaxed);
        rate_limit.start();

        // We consider that the request is made at this point in time, although it
        // is not completeley true:
        // 1. DNS resolution takes time, thus real connection to server happens at
        //    a later time. We could resolve DNS beforehand, cache it's IP address
        //    and reuse that in consecutive requests. But let's assume we have a
        //    niceley set up local DNS resolver that does caching for us.
        // 2. Request building takes time. For some use-cases we could clone
        //    RequestBuilder instead of creating new per each request. In this case
        //    we do not do that because overhead is comparativeley neglible.
        let start = SystemTime::now();

        let result = reqwest::get(&collector.url).await.unwrap();
        // TODO: if status 429 is returned Too many req, then we must check
        // for header Retry-After, to decide when should we retry.

        // Time to result
        // let ttr = SystemTime::now();
        rate_limit.update_from_response(&start, &result);

        let mut ts_next_req = start + req_period;
        // println!("rate_limit: {:?}", rate_limit);
        rate_limit.ts_next_req_adjust(&mut ts_next_req);

        let status = result.status();
        if status == StatusCode::OK {
            let b = result.text().await.unwrap();

            let decoded: DecodedBody = match serde_json::from_str(&b) {
                Ok(val) => val,
                Err(e) => {
                    eprintln!(concat!("ERROR: could not decode response as JSON,",
                        " error: {:?}"
                    ), e);

                    continue;
                }
            };

            let info: PriceInfo = decoded.into();

            // At the moment this is a conscious decission to lose data if our
            // backend can not keep up with incomming data. Because there is no
            // point to buffer too much old data when what we need is real time
            // data.
            if let Err(..) = collector.tx.try_send(info) {
                eprintln!(concat!("ERROR: backend can not process incomming data",
                    " fast enough, dropping packet."
                ));
            }
        }
        else {
            eprintln!("WARNING: remote endpoint returned status: {:?}", status);
        }

        // In this case we could use ttr instead of now, since time taken to parse
        // HTTP response is neglible, but let's assume that it took some time and
        // had more complex structure.
        let now = SystemTime::now();

        // If current process is capable to handle responses fast enough it should
        // have some sleep duration available. If not, then we do not sleep at all
        // and employ best effort processing.
        if ts_next_req > now {
            match ts_next_req.duration_since(now) {
                Ok(sleep_duration) => {
                    sleep(sleep_duration).await;
                }

                // This error should not happen unless the process is running on
                // suspended VM or there is jitter in system time.
                Err(..) => {
                    // On this error, let's trust configuration parameter and assume
                    // processing is neglible in time.
                    sleep(req_period).await;
                }
            }
        }
        else {
            // TODO: we could warn user that system can not keep up with
            // desired setting.
        }

    }
}



/// Basic trait implementation to convert DecodedBody into PriceInfo.
///
/// Currently this only supports BTC/USD conversion and it keeps only 4 decimal
/// digits. And it always expects decimal point.
impl Into<PriceInfo> for DecodedBody {
    fn into(self) -> PriceInfo {
        let rate = self.data.rate_usd;
        // We round down to seconds resolution.
        let timestamp = self.timestamp / 1000;
        let Some(pos) = rate.find('.') else {
            return PriceInfo::new(
                timestamp,
                Symbol::BTC,
                Symbol::USD,
                None,
                0,
            )
        };

        let max_pos = rate.len();
        let digits = if max_pos > pos + 4 {
            4
        }
        else {
            max_pos - pos
        };

        let Ok(val): Result<u64, _> = (&rate[..pos]).parse() else {
            return PriceInfo::new(
                timestamp,
                Symbol::BTC,
                Symbol::USD,
                None,
                0,
            )
        };

        // +1 because decimal separator
        let dec = &rate[pos + 1..pos + digits + 1];
        let Ok(dec): Result<u64, _> = dec.parse() else {
            return PriceInfo::new(
                timestamp,
                Symbol::BTC,
                Symbol::USD,
                None,
                0,
            )
        };

        // This path should execute most of the time.
        if digits == 4 {
            PriceInfo::new(
                timestamp,
                Symbol::BTC,
                Symbol::USD,
                Some(val * 10000 + dec),
                4,
            )
        }
        else {
            let digits: u64 = digits.try_into().unwrap();
            let mul = 10 ^ digits;
            PriceInfo::new(
                timestamp,
                Symbol::BTC,
                Symbol::USD,
                Some(val * mul + dec),
                digits as u8,
            )
        }
    }
}


