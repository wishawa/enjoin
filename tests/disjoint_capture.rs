mod utils;
use utils::YieldFor;

struct Struct<T, U> {
    first: T,
    second: U,
}

#[pollster::test]
async fn capture_struct() {
    let mut s = Struct {
        first: vec![1, 2, 3],
        second: String::from("hello"),
    };
    let b = &s.second;
    enjoin::join_auto_borrow!(
        {
            s.first.push(1);
            YieldFor(3).await;
            s.first.push(1);
        },
        {
            s.first.push(3);
            YieldFor(3).await;
            s.first.push(4);
        }
    );
    drop(b);
}

#[pollster::test]
async fn capture_tuple_nested() {
    let mut a = (1, (2, (4, 5)));
    let b = &a.1 .0;
    enjoin::join_auto_borrow!(
        {
            (&mut a.1 .1).0 += 1;
        },
        {
            a.1 .1 .0 += 1;
        }
    );
    assert_eq!(a, (1, (2, (6, 5))));
    drop(b);
}
