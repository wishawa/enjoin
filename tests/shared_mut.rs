mod utils;
use utils::YieldFor;

#[pollster::test]
async fn shared_number() {
    let mut count = 0;
    enjoin::join_auto_borrow!(
        {
            for _ in 0..5 {
                count += 1;
                YieldFor(1).await;
            }
        },
        {
            for _ in 0..3 {
                count -= 1;
                YieldFor(2).await;
            }
        }
    );
    assert_eq!(count, 2);
}

#[pollster::test]
async fn shared_not_copy() {
    let mut v = vec![0];
    enjoin::join_auto_borrow!(
        {
            for _ in 0..2 {
                v.push(1);
                YieldFor(1).await;
            }
        },
        {
            for _ in 0..3 {
                v.push(1);
                YieldFor(2).await;
            }
        }
    );
    assert_eq!(v, vec![0, 1, 1, 1, 1, 1])
}

#[pollster::test]
async fn no_deref() {
    let mut count = 0;
    enjoin::join_auto_borrow!(
        {
            for _ in 0..5 {
                core::ops::AddAssign::add_assign(&mut count, 1);
                YieldFor(1).await;
            }
        },
        {
            for _ in 0..3 {
                core::ops::SubAssign::sub_assign(&mut count, 1);
                YieldFor(2).await;
            }
        }
    );
    assert_eq!(count, 2);
}
