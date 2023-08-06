#![feature(core_intrinsics)]

use std::intrinsics::prefetch_read_data;

mod future;
mod runtime;

pub use runtime::LocalPool;

#[derive(Debug, Clone)]
pub struct AsyncPrefetch<T> {
    inner: T,
}

impl<T> AsyncPrefetch<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<T> AsyncPrefetch<T> {
    pub async fn prefetch_load<U>(&self) -> &U
    where
        T: AsRef<U>,
    {
        let ptr = self.inner.as_ref() as *const U;
        unsafe { prefetch_read_data(ptr, 0) };
        future::PollOnce::new().await;
        self.inner.as_ref()
    }

    pub fn load<U>(&self) -> &U
    where
        T: AsRef<U>,
    {
        self.inner.as_ref()
    }
}
