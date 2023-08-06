use std::future::Future;

#[pin_project::pin_project]
pub struct PollOnce {
    #[pin]
    polled: bool,
}

impl PollOnce {
    pub fn new() -> Self {
        Self { polled: false }
    }
}

impl Future for PollOnce {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<()> {
        if self.polled {
            return std::task::Poll::Ready(());
        }

        self.project().polled.set(true);
        cx.waker().wake_by_ref();
        std::task::Poll::Pending
    }
}
