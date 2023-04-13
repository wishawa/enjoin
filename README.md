# enjoin

[<img alt="on crates.io" src="https://img.shields.io/crates/v/enjoin?style=for-the-badge" height="20">](https://crates.io/crates/enjoin)
[<img alt="on docs.rs" src="https://img.shields.io/docsrs/enjoin?style=for-the-badge" height="20">](https://docs.rs/enjoin)

**enjoin**'s async join macros operate at the syntax level. It allows you to...

### `break`, `continue`, and `return` out of async code running in a join
```rust
for _ in 0..10 {
    enjoin::join!(
        {
            if do_thing_1().await {
                break;
            }
        },
        {
            if do_thing_2().await {
                continue;
            }
        }
    );
}
```
### Use `?` (try operator) in a join
```rust
async fn fetch_and_save_both() -> Result<(), Box<dyn Error>> {
    enjoin::join!(
        {
            let data = fetch_data_1().await?;
            save_data(data).await?;
        },
        {
            let data = fetch_data_2().await?;
            save_data(data).await?;
        }
    );
}
```
### Share mutable borrows accross a join
... as long as the mutable borrows don't last across yield point / await point.
```rust
let mut count = 0;
enjoin::join_auto_borrow!(
    {
        loop {
            incr_signal.next().await;
            count += 1;
        }
    },
    {
        loop {
            decr_signal.next().await;
            count -= 1;
        }
    }
);
```

See my blog post [here](https://wishawa.github.io/posts/enjoin)
for motivations, working mechanism, comparison to other join macros, and more.