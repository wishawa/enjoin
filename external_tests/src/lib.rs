#[cfg(test)]
mod tests {
    use std::time::Duration;

    use async_std::future;

    async fn wait_for(dur: Duration) {
        use async_std::future::timeout;
        timeout(dur, future::pending::<()>()).await.ok();
    }

    #[async_std::test]
    async fn test_order() {
        let mut record = vec![];
        enjoin::join_auto_borrow!(
            {
                wait_for(Duration::from_millis(10)).await;
                for _ in 0..4 {
                    wait_for(Duration::from_millis(20)).await;
                    record.push(0);
                }
            },
            {
                for _ in 0..4 {
                    wait_for(Duration::from_millis(20)).await;
                    record.push(1);
                }
            }
        );
        assert_eq!(record, vec![1, 0, 1, 0, 1, 0, 1, 0]);
    }
}
