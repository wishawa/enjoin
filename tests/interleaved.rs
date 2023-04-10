mod utils;
use utils::YieldFor;

#[pollster::test]
async fn track_poll() {
    let mut which = Vec::new();
    enjoin::join_auto_borrow!(
        {
            for _ in 0..10 {
                which.push(0);
                YieldFor(1).await;
            }
        },
        {
            for _ in 0..10 {
                which.push(1);
                YieldFor(1).await;
            }
        }
    );
    for (idx, &val) in which.iter().enumerate() {
        assert_eq!(val, idx % 2);
    }
}

#[pollster::test]
async fn track_asymmetric() {
    let mut which = Vec::new();
    enjoin::join_auto_borrow!(
        {
            for _ in 0..3 {
                which.push(0);
                YieldFor(1).await;
            }
        },
        {
            for _ in 0..3 {
                which.push(1);
                YieldFor(2).await;
            }
        }
    );
    assert_eq!(which, [0, 1, 0, 0, 1, 1]);
}
