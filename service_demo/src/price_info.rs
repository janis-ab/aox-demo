


/// Normalized price information structure that can be used for various crypto
/// and currency pairs.
///
/// `base` - this is the first currency that is shown in pair, i.e. BTC/USD,
/// base is BTC.
/// `quote` - is the second shown in pair, i.e. BTC/USD, USD is quote.
/// `rate` - we use whole numbers to store rate because floating point looses
/// precission. Rate contains value that must be divided by 10^decimal.
/// `decimal` - number of decimal numbers in rate value, i.e. if decimal is 3,
/// then rate 20342 real value is 20.342.
#[derive(Debug, Clone)]
pub struct PriceInfo {
    pub timestamp: u64,
    pub base: Symbol,
    pub quote: Symbol,
    pub rate: Option<u64>,
    pub decimal: u8,
}



#[derive(Debug, Clone)]
pub enum Symbol {
    BTC,
    USD,
}



impl PriceInfo {
    pub fn new(timestamp: u64, base: Symbol, quote: Symbol, rate: Option<u64>,
        decimal: u8
    )
        -> Self
    {
        Self {
            timestamp, base, quote, rate, decimal,
        }
    }
}


