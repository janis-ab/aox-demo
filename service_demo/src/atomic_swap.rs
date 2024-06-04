//! Since we need to update information on terminal once per second but
//! incomming data comes into different periods, it is convenient to use atomic
//! data structures.
//!
//! While for current use case we could live with std atomics, i've decided to
//! implement simple example struct that demostrates the use of generics and
//! safe use of unsafe code.
//!
//! This module also demonstrates simple unit test example.



use std::sync::{
    atomic::{
        AtomicPtr,
        Ordering,
    },
};



/// Struct that allows user to swap heap allocated structs atomically.
#[derive(Debug)]
pub struct AtomicSwap<T> {
    ptr: AtomicPtr<T>,
}



impl<T: Send> AtomicSwap<T> {
    pub fn new(ptr: Box<T>) -> Self {
        Self {
            ptr: AtomicPtr::new(Box::<T>::into_raw(ptr)),
        }
    }



    /// Swaps Box pointers atomically.
    pub fn swap(&self, ptr: Box<T>) -> Box<T> {
        let ptr_new = Box::<T>::into_raw(ptr);
        // Use strictest atomic ordering not to have any trouble for different
        // use-cases.
        let ptr_prev = self.ptr.swap(ptr_new, Ordering::SeqCst);

        // This is safe, because we always swap valid and owned pointer.
        unsafe {
            Box::from_raw(ptr_prev)
        }
    }
}



#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug, PartialEq, Clone)]
    struct Test (usize);

    /// A very primitive test to show that swapping works and does not loose
    /// any pointer.
    #[test]
    fn test_atomic_swap() {
        let v1 = Box::new(Test(1));
        let v2 = Box::new(Test(2));
        let v3 = Box::new(Test(3));

        let v1_clone = v1.clone();
        let v2_clone = v2.clone();
        let v3_clone = v3.clone();

        let mut atomic_store = AtomicSwap::new(v1);

        let v1 = atomic_store.swap(v2);
        assert_eq!(v1, v1_clone);

        let v2 = atomic_store.swap(v1);
        assert_eq!(v2, v2_clone);

        let v1 = atomic_store.swap(v3);
        assert_eq!(v1, v1_clone);
    }

}


