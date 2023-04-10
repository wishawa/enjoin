mod utils;
use utils::YieldFor;

#[pollster::test]
async fn basic_try_join_successful() {
    let res = '_ok: {
        let _ok = enjoin::join!(
            {
                YieldFor(3).await;
                3
            },
            {
                YieldFor(4).await;
                "hello"
            },
            {
                YieldFor(2).await;
                "hi"
            }
        );
        true
    };
    assert!(res);
}

#[pollster::test]
async fn basic_try_join_fail() {
    let res = 'ok: {
        let _ok = enjoin::join!(
            {
                YieldFor(3).await;
                3
            },
            {
                YieldFor(4).await;
                if 1 + 1 == 2 {
                    break 'ok false;
                }
            },
            {
                YieldFor(2).await;
                "hi"
            }
        );
        panic!("nope");
    };
    assert!(!res);
}
