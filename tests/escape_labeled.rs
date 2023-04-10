#[pollster::test]
async fn labeled_break_one() {
    let mut x = 0;
    'a: for i in 0..5 {
        enjoin::join!(
            {
                if i >= 2 {
                    break 'a;
                }
            },
            {}
        );
        x += 1;
    }
    assert_eq!(x, 2);
}

#[pollster::test]
async fn labeled_break_nested() {
    let mut x = 0;
    'a: for i in 0..5 {
        for _ in 0..2 {
            enjoin::join!({}, {
                if i <= 2 {
                    continue 'a;
                }
            });
            x += 1;
        }
    }
    assert_eq!(x, 4);
}

#[pollster::test]
async fn labeled_break_nested_through() {
    let mut x = 0;
    'a: for i in 0..5 {
        enjoin::join!({
            for j in 0..5 {
                if j <= 1 {
                    continue;
                }
                if i <= 2 {
                    continue 'a;
                }
            }
        });
        x += 1;
    }
    assert_eq!(x, 2);
}
