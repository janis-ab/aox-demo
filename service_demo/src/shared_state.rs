use std::sync::atomic::AtomicUsize;



#[derive(Default)]
pub struct SharedState {
    // While this is 0, all threads and coroutines can continue to do their
    // tasks. When this is set to 1, concurrent tasks should be gracefully
    // finished.
    // If some synchronous task is sleeping, it will take this into account only
    // when the thread is waken up.
    pub shut_down: AtomicUsize,
}


