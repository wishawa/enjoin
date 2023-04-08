use enjoin::join;

mod utils;
use utils::{NotCopy, YieldOnce};

#[pollster::test]
async fn one() {
    let mut data = 1;
    let x = 'a: {
        join!({
            data += 1;
            break 'a 4;
        });
        3
    };
    assert_eq!(x, 4);
    assert_eq!(data, 2);
}

#[pollster::test]
async fn questionmark() {
    async fn inner() -> Result<i32, &'static str> {
        join!({
            Ok(7)?;
            Err("haha")?;
            Ok(6)?;
            Ok(3)
        })
        .0
    }
    let res = inner().await;
    assert_eq!(res, Err("haha"));
}

#[pollster::test]
async fn not_copy() {
    let mut data = NotCopy(1);
    join!(
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
    join!(
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
    join!(
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
    join!(
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
// 	join!(
// 		{
// 			data += 1;
// 		}
// 	);
// 	assert_eq!(*borrow, 2);
// }

#[pollster::test]
async fn has_loop() {
    let mut shared = 0;
    join! {
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
