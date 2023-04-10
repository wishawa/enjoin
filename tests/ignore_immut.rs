#[pollster::test]
async fn ignore_immut_reference() {
    let count = 0i32;
    let a = &count;
    enjoin::join_auto_borrow!(
        {
            let _b = &count;
        },
        {
            let _b = (&count).pow(2);
        }
    );
    drop(a);
}
