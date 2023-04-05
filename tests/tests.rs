use enjoin::enjoin;
use futures::Future;

struct YieldOnce(bool);
impl YieldOnce {
    fn new() -> Self {
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

struct NotCopy(i32);
impl NotCopy {
    fn add(&mut self, other: i32) {
        self.0 += other;
    }
}

#[pollster::test]
async fn one() {
    let mut data = 1;
    enjoin!({
        data += 1;
    },);
    assert_eq!(data, 2);
}

#[pollster::test]
async fn not_copy() {
    let mut data = NotCopy(1);
    enjoin!(
        {
            data.add(5);
        },
        {
            data.add(-3);
        }
    );
    assert_eq!(data.0, 3);
}

#[pollster::test]
async fn two() {
    let mut data = 1;
    enjoin!(
        {
            data += 1;
        },
        {
            data -= 2;
        }
    );
    assert_eq!(data, 0);
}

#[pollster::test]
async fn disjoint_tuple() {
    let mut data = (2, 3);
    let borrow = &data.1;
    enjoin!(
        {
            data.0 += 1;
        },
        {
            data.0 -= 1;
        }
    );
    assert_eq!(*borrow, 3);
    assert_eq!(data.0, 2);
}

#[pollster::test]
async fn disjoint_tuple_nested() {
    let mut data = (2, (3, 4), ("asdf", (1, 7)));
    enjoin!(
        {
            data.2 .1 .1 += 1;
        },
        {
            data.2 .1 .0 -= 1;
            data.2 .1 .1 -= 1;
        }
    );
    assert_eq!(data.2 .1 .1, 7);
    assert_eq!(data.2 .1 .0, 0);
}

// #[pollster::test]
// async fn compile_fail_1() {
// 	let mut data = 1;
// 	let borrow = &data;
// 	enjoin!(
// 		{
// 			data += 1;
// 		}
// 	);
// 	assert_eq!(*borrow, 2);
// }

#[pollster::test]
async fn has_loop() {
    let mut shared = 0;
    enjoin! {
        {
            for _ in 0..5 {
                shared += 1;
                YieldOnce::new().await;
            }
        },
        {
            for _ in 0..2 {
                YieldOnce::new().await;
                shared -= 1;
            }
        }
    };
    assert_eq!(shared, 3);
}
