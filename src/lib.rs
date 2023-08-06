mod future;
mod runtime;

pub use future::PollOnce;
pub use runtime::LocalPool;

#[cfg(target_arch = "x86_64")]
pub fn prefetch<T>(reference: &T) {
    use std::arch::x86_64::{_mm_prefetch, _MM_HINT_NTA};
    let pointer: *const _ = &*reference;
    unsafe { _mm_prefetch(pointer as _, _MM_HINT_NTA) }
}

pub async fn async_load<T>(r: &T) -> &T {
    prefetch(r);
    future::PollOnce::new().await;
    r
}
