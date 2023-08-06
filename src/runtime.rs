use std::collections::VecDeque;

use futures::{Future, FutureExt};

pub struct LocalPool<F> {
    queue: VecDeque<F>,
}

impl<F: Future + Unpin> LocalPool<F> {
    pub fn new() -> Self {
        Self {
            queue: Default::default(),
        }
    }

    pub fn add_future(&mut self, future: F) {
        self.queue.push_back(future);
    }

    pub fn run_until_finish(&mut self) {
        while let Some(mut fut) = self.queue.pop_front() {
            match fut.poll_unpin(&mut futures::task::Context::from_waker(
                futures::task::noop_waker_ref(),
            )) {
                std::task::Poll::Pending => self.queue.push_back(fut),
                std::task::Poll::Ready(_) => {}
            }
        }
    }
}
