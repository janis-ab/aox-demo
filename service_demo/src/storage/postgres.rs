use std::{
    env,
    sync::{
        Arc,
        atomic::Ordering,
    }
};

use tokio::sync::mpsc;
use tokio_postgres::{
    connect as pg_connect,
    NoTls,
    Client,
};

use async_trait::async_trait;

use crate::{
    ohlc::Ohlc,
    shared_state::SharedState,
    storage::Storage,
};



/// Postgres storage implementation.
///
/// `rx` - receiver for storage channel.
/// `count` - count for received Ohlc items.
pub struct Postgres {
    rx: mpsc::Receiver<Ohlc>,
    count: usize,
    config_string: Option<String>,
    client: Option<Client>,
}



macro_rules! env_load_or_default {
    ($name:expr, $default:expr) => {{
        match env::var($name) {
            Ok(val) => val,
            Err(..) => $default.to_string(),
        }
    }}
}



impl Postgres {
    pub fn new(rx: mpsc::Receiver<Ohlc>) -> Self {
        Self {
            rx,
            count: 0,
            config_string: None,
            client: None,
        }
    }



    /// Load configuration from ENV.
    fn config_string_load(&mut self) {
        if !self.config_string.is_none() {
            return
        }

        let host = env_load_or_default!("DB_HOST", "127.0.0.1");
        let port = env_load_or_default!("DB_PORT", "5432");
        let user = env_load_or_default!("DB_USER", "demouser");
        let dbname = env_load_or_default!("DB_NAME", "demo");
        let password = env_load_or_default!("DB_PASS", "");

        let con_cfg = format!(concat!("host='{}' port='{}' user='{}'",
            " dbname='{}' password='{}'"), host, port, user, dbname, password
        );

        self.config_string = Some(con_cfg);
    }



    // Method that ensures that there is active Postgresql connection.
    async fn connection_ensure(&mut self) {
        if let Some(..) = &self.client { return }

        self.config_string_load();
        let Some(ref con_cfg) = self.config_string else {
            return;
        };

        // println!("Spawning new postgres connection.");
        let (client, connection) = match pg_connect(con_cfg, NoTls).await {
            Ok(ret) => ret,
            Err(e) => {
                eprintln!("ERROR: could not connect to DB: {:?}", e);
                return;
            }
        };

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("ERROR: Postgres connection error: {:?}", e);
            }
        });

        self.client = Some(client);
    }



    // Insert OHLC information into Postgresql DB if connection is available.
    async fn insert_ohlc(&mut self, ohlc: Ohlc) -> Result<(), ()> {
        self.connection_ensure().await;

        let Some(ref client) = self.client else {
            eprintln!("ERROR: DB connection not active.");
            return Err(())
        };

        let sql = r#"
            insert into ohlc(start, open, high, low, close, duration)
            values(to_timestamp($1::bigint), $2, $3, $4, $5, $6)
        "#;

        let r = client.query(sql, &[
            &(ohlc.start as i64), &(ohlc.open as i64), &(ohlc.high as i64),
            &(ohlc.low as i64), &(ohlc.close as i64), &(ohlc.duration as i32),
        ]).await;

        if let Err(e) = r {
            eprintln!("ERROR: Database insert failed, error: {:?}", e);
            return Err(())
        }

        return Ok(())
    }
}



#[async_trait]
impl Storage for Postgres {
    async fn main(mut self, shared_state: Arc<SharedState>) {
        // If collector thread has crashed, this thread has no use to be alive.
        while let Some(ohlc) = self.rx.recv().await {
            self.count += 1;

            if let Err(..) = self.insert_ohlc(ohlc).await {
                // TODO: here we could implement retry insert policy based on
                // specific usecase, i.e. if incomming channel is not full,
                // retry, if it is full, then drop row so that we get real time
                // data.
            }

            let intr = shared_state.shut_down.load(Ordering::Relaxed);
            if intr != 0 {
                return
            }
        }
    }
}


