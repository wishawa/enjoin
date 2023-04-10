mod utils;
use utils::YieldFor;

#[pollster::test]
async fn race() {
    #[allow(unreachable_code)]
    let res = 'race: {
        enjoin::join!(
            {
                YieldFor(4).await;
                break 'race 4;
            },
            {
                YieldFor(6).await;
                break 'race 6;
            },
            {
                YieldFor(20).await;
                break 'race 20;
            },
            {
                YieldFor(3).await;
                break 'race 3;
            }
        );
        panic!("NO");
    };
    assert_eq!(res, 3)
}
