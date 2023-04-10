mod utils;

#[pollster::test]
async fn unlabeled_break_while() {
    let mut x = 0;
    while x < 20 {
        enjoin::join!(
            {
                if x >= 2 {
                    break;
                }
            },
            {}
        );
        x += 1;
    }
    assert_eq!(x, 2);
}

#[pollster::test]
async fn unlabeled_break_loop() {
    let mut x = 0;
    let q = loop {
        enjoin::join!(
            {
                if x >= 2 {
                    break 6;
                }
            },
            {
                if false {
                    break 23;
                }
            }
        );
        x += 1;
    };
    assert_eq!(q, 6);
    assert_eq!(x, 2);
}

#[pollster::test]
async fn unlabeled_continue_for() {
    let mut x = 0;
    for i in 0..5 {
        enjoin::join!({}, {
            if i <= 2 {
                continue;
            }
        });
        x += 1;
    }
    assert_eq!(x, 2);
}

#[pollster::test]
async fn return_out() {
    enjoin::join!(
        {
            utils::YieldFor(3).await;
            if true {
                return;
            }
        },
        {}
    );
    panic!();
}

#[pollster::test]
async fn unlabeled_break_shadowed() {
    let mut x = 0;
    for _ in 0..5 {
        enjoin::join!({
            for i in 0..5 {
                if i > 3 {
                    break;
                }
                x += 1;
            }
        });
    }
    assert_eq!(x, 20);
}
