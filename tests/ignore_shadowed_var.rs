#[pollster::test]
async fn let_shadowed() {
    let x = 3;
    enjoin::join_auto_borrow!(
        {
            let mut x = 400;
            x += 1;
            assert_eq!(x, 401);
        },
        {
            let mut x = 400;
            x += 1;
            assert_eq!(x, 401);
        },
    );
    assert_eq!(x, 3);
}

#[pollster::test]
async fn let_not_shadowed() {
    let mut x = 3;
    enjoin::join_auto_borrow!(
        {
            let mut x = {
                x += 1;
                x
            };
            x += 100;
            drop(x);
        },
        {
            let mut x = {
                x += 2;
                x
            };
            x += 100;
            drop(x);
        }
    );
    assert_eq!(x, 6);
}

#[pollster::test]
async fn if_let_shadowed() {
    let x = 3;
    enjoin::join_auto_borrow!(
        {
            if let Some(mut x) = None::<i32> {
                x += 4;
                drop(x);
            }
        },
        {
            if let Some(mut x) = None::<i32> {
                x += 4;
                drop(x);
            }
        }
    );
    assert_eq!(x, 3);
}

#[pollster::test]
async fn if_let_not_shadowed() {
    let mut x = 3;
    enjoin::join_auto_borrow!(
        {
            if let Some(mut x) = Some({
                x += 1;
                x
            }) {
                x += 1;
                drop(x);
            }
        },
        {
            if let Some(mut x) = Some({
                x += 10;
                x
            }) {
                x += 10;
                drop(x);
            }
        }
    );
    assert_eq!(x, 14);
}

#[pollster::test]
async fn for_shadowed() {
    let x = 3;
    enjoin::join_auto_borrow!(
        {
            for mut x in 0..10 {
                x += 1;
                drop(x);
            }
        },
        {
            for mut x in 0..10 {
                x += 1;
                drop(x);
            }
        }
    );
    drop(x);
}

#[pollster::test]
async fn for_not_shadowed() {
    let mut x = 3;
    enjoin::join_auto_borrow!(
        {
            for mut x in {
                x += 1;
                x
            }..10
            {
                x += 5;
                let _ = x + 2;
            }
        },
        {
            for x in {
                x += 2;
                x
            }..10
            {
                let _ = x + 1;
            }
        }
    );
    assert_eq!(x, 6);
}

#[pollster::test]
async fn match_shadowed() {
    let x = 3;
    enjoin::join_auto_borrow!(
        {
            match Some(5) {
                Some(mut x) => {
                    x += 1;
                    drop(x);
                }
                _ => {}
            }
        },
        {
            match Some(5) {
                Some(x) => {
                    drop(x);
                }
                _ => {}
            }
        }
    );
    drop(x);
}

#[pollster::test]
async fn block_released() {
    let mut x = 3;
    enjoin::join_auto_borrow!(
        {
            {
                let x = 5;
                drop(x);
            }
            x += 1;
        },
        {
            {
                let x = 5;
                drop(x);
            }
            x += 1;
        }
    );
    assert_eq!(x, 5);
}

#[pollster::test]
async fn nested_block_released() {
    let mut x = 3;
    enjoin::join_auto_borrow!(
        {
            {
                let x = 4;
                {
                    let mut x = 6;
                    x -= 2;
                    drop(x);
                }
                let _b = x + 1;
            }
            x -= 1;
        },
        {
            {
                let x = 4;
                {
                    let mut x = 6;
                    x -= 2;
                    drop(x);
                }
                let _b = x + 1;
            }
            x -= 1;
        }
    );
    assert_eq!(x, 1);
}
