#[pollster::test]
async fn sample_expansion() {
    async fn do_thing_a() -> i32 {
        20
    }
    async fn do_thing_b() -> Option<i32> {
        None
    }
    async fn inner() -> Option<&'static str> {
        let mut done = 0;
        'a: {
            enjoin::join_auto_borrow!(
                {
                    let res = do_thing_a().await;
                    if res > 3 {
                        return Some("hello");
                    } else {
                        done += 1;
                        if done == 2 {
                            break 'a;
                        }
                    }
                },
                {
                    let _res = do_thing_b().await?;
                    done += 1;
                }
            );
        };
        None
    }
    assert_eq!(inner().await, Some("hello"));
}
