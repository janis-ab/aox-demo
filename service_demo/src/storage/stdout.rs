use std::sync::{
    Arc,
    atomic::Ordering,
};

use tokio::sync::mpsc;

use async_trait::async_trait;

use crate::{
    ohlc::Ohlc,
    shared_state::SharedState,
    storage::Storage,
};



/// Proof of concept storate that does not store anything, but outputs data
/// to STDOUT instead.
pub struct Stdout {
    rx: mpsc::Receiver<Ohlc>,
}



impl Stdout {
    pub fn new(rx: mpsc::Receiver<Ohlc>) -> Self {
        Self {
            rx,
        }
    }
}



#[async_trait]
impl Storage for Stdout {
    async fn main(mut self, shared_state: Arc<SharedState>) {
        // If collector thread has crashed, this thread has no use to be alive.
        while let Some(ohlc) = self.rx.recv().await {
            println!("{:?}", ohlc);

            let intr = shared_state.shut_down.load(Ordering::Relaxed);
            if intr != 0 {
                return
            }
        }
    }
}


