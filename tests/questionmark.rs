#[pollster::test]
async fn try_continue() {
    async fn inner() -> Result<String, String> {
        enjoin::join!({
            Ok::<_, String>("haha".to_string())?;
            Ok("yes".to_string())
        })
        .0
    }
    assert_eq!(inner().await, Ok("yes".to_string()));
}

#[pollster::test]
async fn try_break() {
    async fn inner() -> Result<String, String> {
        enjoin::join!({
            Ok::<_, String>("haha".to_string())?;
            Err::<String, _>("oops".to_string())?;
            Ok::<_, String>("no".to_string())
        })
        .0
        .ok();
        panic!("NO");
    }
    assert_eq!(inner().await, Err("oops".to_string()));
}
