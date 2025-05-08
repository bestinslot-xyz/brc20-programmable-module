use std::error::Error;
use std::sync::{RwLock, RwLockReadGuard};

pub struct SharedData<T> {
    inner: RwLock<T>,
}

/// This struct provides a thread-safe way to share data between threads.
/// It uses a read-write lock to allow multiple readers or a single writer at a time.
/// Reads return a read lock guard, while writes return a result of the operation.
///
/// If/when the lock is poisoned, reads are still possible to make sure the server
/// can continue to run.
impl<T> SharedData<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: RwLock::new(inner),
        }
    }

    /// This method returns a read lock guard that can be used to access the inner data.
    ///
    /// Avoid storing the read lock guard outside of this function,
    /// as it may lead to deadlocks if the lock is held for too long.
    ///
    /// In case this is needed to be stored and/or used outside of the function,
    /// it is recommended to use the `read_fn` method instead.
    pub fn read(&self) -> RwLockReadGuard<T> {
        match self.inner.read() {
            Ok(guard) => guard,
            Err(error) => error.into_inner(),
        }
    }

    /// This method allows you to read from the inner data and handle errors.
    /// It returns a result of the operation.
    pub fn read_fn<F, R>(&self, f: F) -> Result<R, Box<dyn Error>>
    where
        F: FnOnce(&T) -> Result<R, Box<dyn Error>>,
    {
        let guard = match self.inner.read() {
            Ok(guard) => guard,
            Err(error) => error.into_inner(),
        };
        f(&*guard)
    }

    /// This method allows you to read from the inner data and handle errors.
    pub fn write_fn<F, R>(&self, f: F) -> Result<R, Box<dyn Error>>
    where
        F: FnOnce(&mut T) -> Result<R, Box<dyn Error>>,
    {
        let mut guard = self.inner.write().expect("Failed to acquire write lock");
        f(&mut guard)
    }

    /// This method allows you to write to the inner data without checking for errors.
    pub fn write_fn_unchecked<F>(&self, f: F)
    where
        F: FnOnce(&mut T) -> (),
    {
        let mut guard = self.inner.write().expect("Failed to acquire write lock");
        f(&mut guard)
    }
}
