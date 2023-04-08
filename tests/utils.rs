use core::future::Future;

pub struct YieldOnce(bool);
impl YieldOnce {
    pub fn new() -> Self {
        Self(false)
    }
}

impl Future for YieldOnce {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.get_mut();
        if this.0 {
            std::task::Poll::Ready(())
        } else {
            this.0 = true;
            cx.waker().wake_by_ref();
            std::task::Poll::Pending
        }
    }
}

pub struct NotCopy(pub i32);
impl NotCopy {
    pub fn add(&mut self, other: i32) {
        self.0 += other;
    }
}
