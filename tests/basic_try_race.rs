mod utils;
use utils::YieldFor;

#[pollster::test]
async fn try_race_successful() {
    let a = 'race: {
        let errs = enjoin::join!(
            {
                YieldFor(3).await;
                if 1 < 2 {
                    break 'race Ok("yes");
                }
            },
            {
                YieldFor(1).await;
                if 2 < 1 {
                    break 'race Ok("yes2");
                } else {
                    "no"
                }
            }
        );
        Err(errs)
    };
    assert_eq!(a, Ok("yes"));
}

#[pollster::test]
async fn try_race_fail() {
    let a = 'race: {
        let errs = enjoin::join!(
            {
                YieldFor(3).await;
                if false {
                    break 'race Ok("good");
                }
                "no"
            },
            {
                YieldFor(3).await;
                "another no"
            }
        );
        Err(errs)
    };
    assert_eq!(a, Err(("no", "another no")));
}
