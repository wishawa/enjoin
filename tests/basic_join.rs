mod utils;
use utils::YieldFor;

#[pollster::test]
async fn join_1() {
    let (o1,) = enjoin::join_auto_borrow!({
        YieldFor(3).await;
        42
    });
    assert_eq!(o1, 42);
}

#[pollster::test]
async fn join_2() {
    let (o1, o2) = enjoin::join_auto_borrow!(
        {
            YieldFor(3).await;
            42
        },
        {
            YieldFor(3).await;
            "lol"
        }
    );
    assert_eq!(o1, 42);
    assert_eq!(o2, "lol");
}

#[pollster::test]
async fn join_3() {
    let (o1, o2, o3) = enjoin::join_auto_borrow!(
        {
            YieldFor(3).await;
            42
        },
        {
            YieldFor(3).await;
            "lol"
        },
        {
            YieldFor(3).await;
            vec![1, 2, 3]
        }
    );
    assert_eq!(o1, 42);
    assert_eq!(o2, "lol");
    assert_eq!(o3, vec![1, 2, 3]);
}
