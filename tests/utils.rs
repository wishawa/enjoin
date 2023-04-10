use core::future::Future;

pub struct YieldFor(pub usize);

impl Future for YieldFor {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.get_mut();
        if this.0 == 0 {
            std::task::Poll::Ready(())
        } else {
            this.0 -= 1;
            cx.waker().wake_by_ref();
            std::task::Poll::Pending
        }
    }
}
