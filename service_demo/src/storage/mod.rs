pub mod postgres;
pub mod stdout;

use std::sync::Arc;
use crate::shared_state::SharedState;
use async_trait::async_trait;



/// Storage traits should implement this method, so that they can be run in
/// separate async function.
#[async_trait]
pub trait Storage {
    async fn main(self, shared_state: Arc<SharedState>);
}



/// Nice wrapper to abstract away Storage trait.
pub async fn main(storage: impl Storage, shared_state: Arc<SharedState>) {
    storage.main(shared_state).await
}


