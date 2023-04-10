#[pollster::test]
async fn block_shadowed() {
    let o = 'a: {
        if false {
            break 'a (9,);
        }
        enjoin::join!({
            'a: {
                break 'a 3;
            };
            6
        })
    };
    assert_eq!(o.0, 6);
}
