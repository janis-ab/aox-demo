use std::sync::{
    Arc,
    atomic::Ordering,
};

use tokio::sync::mpsc;

use crate::{
    shared_state::SharedState,
    price_info::{
        PriceInfo,
    },
    atomic_swap::AtomicSwap,
    ohlc::Ohlc,
};



pub struct OhlcCalc {
    rx: mpsc::Receiver<PriceInfo>,
    tx_storage: mpsc::Sender<Ohlc>,
    terminal: Arc<AtomicSwap<Option<Ohlc>>>,
}



impl OhlcCalc {
    pub fn new(rx: mpsc::Receiver<PriceInfo>, tx_storage: mpsc::Sender<Ohlc>,
        terminal: Arc<AtomicSwap<Option<Ohlc>>>
    )
        -> Self
    {
        Self {
            rx, tx_storage, terminal,
        }
    }
}



pub async fn main(mut calc: OhlcCalc, shared_state: Arc<SharedState>) {
    let mut ohlc = Ohlc::default();

    let mut terminal_ohlc: Box<Option<Ohlc>> = Box::new(None);

    // If collector thread has crashed, this thread has no use to be alive.
    while let Some(info) = calc.rx.recv().await {
        let Some(rate) = info.rate else { continue };

        if info.decimal != 4 {
            todo!("must normalize incomming data before using it in calculations")
        }

        let ts_unix_minute = info.timestamp - (info.timestamp % 60);

        // If new minute has started, reset values and store current into DB.
        if ohlc.start != ts_unix_minute {
            let ohlc_prev = ohlc;

            ohlc = Ohlc::default();
            ohlc.start = ts_unix_minute;
            ohlc.duration = 60;
            ohlc.open = rate;
            ohlc.low = rate;
            ohlc.high = rate;
            ohlc.close = rate;

            if ohlc_prev.start != 0 {
                // Loose data if DB backend can not keep up.
                if let Err(..) = calc.tx_storage.try_send(ohlc_prev) {
                    eprintln!(concat!("Storage backend can not keep up with",
                        " generated data. dropping Ohlc."
                    ));
                }

            }
        }
        // Update OHLC values accordingly.
        else {
            if ohlc.high < rate {
                ohlc.high = rate;
            }

            if ohlc.low > rate {
                ohlc.low = rate;
            }

            // Any value is considered a close, because we do not know if we
            // will get data for the same minute in next message.
            ohlc.close = rate;
        }

        *terminal_ohlc = Some(ohlc.clone());
        terminal_ohlc = calc.terminal.swap(terminal_ohlc);

        let intr = shared_state.shut_down.load(Ordering::Relaxed);
        if intr != 0 {
            return
        }
    }
}


