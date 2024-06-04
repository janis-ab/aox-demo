use std::{
    sync::{
        Arc,
        atomic::Ordering,
    },
    env,
};

use dotenv;
use tokio::sync::mpsc;

pub mod async_http_collector;
pub mod shared_state;
pub mod rate_limit;
pub mod price_info;
pub mod ohlc_calc;
pub mod ohlc;
pub mod storage;
pub mod atomic_swap;
pub mod terminal_output;

use async_http_collector::AsyncHTTPCollector;
use shared_state::SharedState;
use price_info::PriceInfo;
use ohlc_calc::OhlcCalc;
use ohlc::Ohlc;
use storage::postgres::Postgres;
use atomic_swap::AtomicSwap;
use terminal_output::TerminalOutput;



#[tokio::main]
async fn main() {
    match dotenv::from_path(".env") {
        Ok(()) => {},
        Err(e) => {
            eprintln!("ERROR: could not load .env, error: {:?}", e);
            return
        }
    }

    let Ok(url_rates) = env::var("URL_RATES") else {
        eprintln!("ERROR: URL_RATES must be configured in .env file.");
        return
    };

    let state = Arc::new(SharedState::default());

    let boxed_ohlc = Box::new(Some(Ohlc::default()));
    let terminal_ohlc = Arc::new(AtomicSwap::new(boxed_ohlc));

    let (tx, rx) = mpsc::channel::<PriceInfo>(200);
    let (tx_storage, rx_storage) = mpsc::channel::<Ohlc>(200);

    // TODO: for now we just build a single collector, but technically we could
    // fork multiple collectors for multiple crypto rates based on .env config.
    let url = format!("{}/bitcoin", url_rates);
    let mut collector = AsyncHTTPCollector::new(&url, tx);
    collector.request_period_millis_set(800);

    let calc = OhlcCalc::new(rx, tx_storage, terminal_ohlc.clone());

    // TODO: here based on configuration we could choose different storage
    // implementation, like MongoDB, Redis, CSV file, etc.
    let storage = Postgres::new(rx_storage);
    // let storage = storage::stdout::Stdout::new(rx_storage);

    let terminal = TerminalOutput::new(terminal_ohlc);

    let collector_h = tokio::spawn(
        async_http_collector::main(collector, state.clone())
    );
    let calc_h = tokio::spawn(ohlc_calc::main(calc, state.clone()));
    let storage_h = tokio::spawn(storage::main(storage, state.clone()));
    let terminal_h = tokio::spawn(terminal_output::main(terminal, state.clone()));

    let state_signal = state.clone();
    let sig_h = tokio::spawn(async move {
        if let Ok(..) = tokio::signal::ctrl_c().await {
            state_signal.shut_down.store(1, Ordering::SeqCst);
        }
    });

    // TODO: implement better task handler join and signal handling. At the
    // moment this is not so important, but for real-time production processes,
    // graceful shutdown is important. But how errors are handled is highly
    // dependent on process manager, i.e. docker can automatically restart
    // process if it has exited, so we don't have to write complex code here.
    // Some other process manager might have different behavior, so in those
    // cases we should implement more complex code here that is able to recover
    // process from partially crashed state.

    let _ = collector_h.await;
    state.shut_down.store(1, Ordering::Relaxed);

    let _ = calc_h.await;
    let _ = storage_h.await;
    let _ = terminal_h.await;
    let _ = sig_h.await;
}


