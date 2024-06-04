


/// Stores Open-High-Low-Close price information per specified duration.
///
/// Ohlc always uses 4 decimal places. It is hardcoded.
///
/// `duration` - used to define Ohlc time duration in seconds so that OHLC can
/// represent various calculation durations like, 1 min, 5 min, 1 hour, etc.
/// I.e. 1 minute duration = 60, start will be a round Unix timestamp for
/// specified minute.
#[derive(Default, Debug, Clone)]
pub struct Ohlc {
    pub start: u64,
    pub open: u64,
    pub high: u64,
    pub low: u64,
    pub close: u64,
    pub duration: u32,
}


