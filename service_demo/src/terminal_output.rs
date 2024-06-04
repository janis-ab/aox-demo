
use std::{
    sync::{
        Arc,
        atomic::Ordering,
    },
    time::Duration,
};



use tokio::{
    time::sleep,
};

use crate::{
    shared_state::SharedState,
    atomic_swap::AtomicSwap,
    ohlc::Ohlc,
};



pub struct TerminalOutput {
    terminal: Arc<AtomicSwap<Option<Ohlc>>>,
}



impl TerminalOutput {
    pub fn new(terminal: Arc<AtomicSwap<Option<Ohlc>>>) -> Self {
        Self {
            terminal,
        }
    }
}



pub async fn main(term: TerminalOutput, shared_state: Arc<SharedState>) {
    // Terminal update is 1 per second as requested in specification.
    // TODO: we could make this configurabel from .env.
    let sleep_duration = Duration::from_millis(1000);

    let mut intr = shared_state.shut_down.load(Ordering::Relaxed);

    let mut ohlc: Box<Option<Ohlc>> = Box::new(None);
    let mut ohlc_display: Option<Ohlc> = None;

    while intr == 0 {
        intr = shared_state.shut_down.load(Ordering::Relaxed);

        ohlc = term.terminal.swap(ohlc);

        // Set memory to None, so that we do not print old Ohlc info in case
        // if collector is retrieving data slower than 1 per sec or data
        // calculator can not calculate fast enough rate.
        if let Some(..) = *ohlc {
            ohlc_display = *ohlc;
            *ohlc = None;
        }

        if let Some(ref ohlc) = ohlc_display {
            if ohlc.start != 0 {
                println!("{:?}", ohlc);
            }
        }
        sleep(sleep_duration).await;
    }
}


