//! *enjoin* - async joining at the syntax level
//!
//! The macros provided in this crate take blocks of code and run them
//! concurrently. The macros try to make the blocks work in the same way that
//! regular blocks (that are not being run concurrently) work.
//!
//! ## Syntax
//! ```
//! # async {
//! let (res_1, res_2) = enjoin::join!(
//!     {
//!         // Code goes here
//!     },
//!     {
//!         // Code goes here
//!     }
//! );
//! # };
//! ```
//!
//! The macro takes blocks of code and execute them concurrently.
//! Any number of blocks may be supplied.
//!
//! The blocks are regular blocks, not async blocks.
//! The blocks can contain async code (i.e. you can use `.await` in the blocks).
//!
//! The results are returned as a tuple.
//!
//! ## Features
//!
//! This features are things that you can already do in regular blocks.
//! They are being listed here because they don't work in async blocks
//! (which other join implementations rely on).
//!
//! ### Branching statements support
//!
//! You can use `break`, `continue`, and `return` inside the blocks.
//!
//! ```
//! # async {
//! let x = 'a: loop {
//!     enjoin::join!(
//!         {
//!             // do something
//!             break 'a 5;
//!         },
//!         {
//!             // do something else
//!             continue;
//!         },
//!         {
//!             // do something
//!             return;
//!         }
//!     );
//! };
//! # };
//! ```
//!
//! The `break`/`continue` may be labeled or unlabeled.
//! `break`-ing with a value is supported.
//!
//! You can, of course, still use `break` and `continue` for
//! loops or blocks contained inside the join macro.
//!
//! As in regular Rust, unlabeled `break`/`return` effects
//! the innermost loop,
//! and labeled `break`/`return` effects the innermost loop with that label.
//!
//! Returning from inside the join will cause the current function to return.
//!
//! If the branching statement causes execution to jump out of the macro,
//! all the code executing in the macro will be stopped and dropped.
//!
//! ### Try operator (`?`) support
//!
//! You can use the `?` operator inside your blocks just as you would outside
//! a join macro.
//!
//! ```
//! async fn f() -> Result<i32, &'static str> {
//!     enjoin::join!(
//!         {
//!             let a = Ok(5);
//!             let b = Err("oh no");
//!             let c = a?;
//!             b?; // causes the `f` function to return `Err("oh no")`
//!         },
//!         {
//!             // ...
//!         }
//!     );
//!     unreachable!("would have returned error at `b?`")
//! }
//! ```
//!
//! ### Shared borrowing support
//!
//! If two or more blocks mutably borrow the same value, the `join_auto_borrow!`
//! macro will automatically put that value in a RefCell.
//! (`join_auto_borrow!` also do everything `join!` does)
//!
//! ```
//! # async {
//! let mut count = 0;
//! enjoin::join_auto_borrow!(
//!     {
//!         // ...
//!         count += 1;
//!     },
//!     {
//!         count -= 1;
//!     }
//! );
//! # };
//! ```
//!
//! The macro makes sure the RefCell will never panic by disallowing
//! shared borrows from lasting across await yieldpoints.
//!
//! ```compile_fail
//! # async {
//! let mut count = 0;
//! enjoin::join_auto_borrow!(
//!     {
//!         let borrow = &mut count;
//!         std::future::ready(123).await; // <- borrow crosses this yieldpoint
//!         drop(borrow);
//!     },
//!     {
//!         count += 1;
//!     }
//! );
//! # };
//! ```
//!
//! ## More information
//!
//! There is [a blog post](https://wishawa.github.io/posts/enjoin) detailing
//! why this macro was made and how it works.
//!
//! The source code is [here on GitHub](https://github.com/wishawa/enjoin).
//!
//! ## Troubleshooting
//!
//! * If branching statements and/or captured variables are hidden
//!     in another macro, *enjoin* wouldn't be able to transform them.
//!     This will usually cause compilation failure.
//!
//! 	```rust
//!     # async {
//! 	enjoin::join!({ vec![
//!         1, 2, 3
//! 		// enjoin can't see the code in this vec!
//! 	] });
//!     # };
//! 	```
//!
//! ---
//!
//! * If an `await` is hidden inside a macro, `join_auto_borrow!` won't be able
//!     to unlock the RefCell for the yieldpoint, leading to a RefCell panic.
//!     This limitation means you can't nest `enjoin::join!` or `tokio::join!`
//!     within `enjoin::join_auto_borrow!`.
//!
//! ---
//!
//! * With only syntactic information, *enjoin* can only guess whether or not a
//!     name is a borrowed variable, and whether or not that borrow is mutable.
//!     We have heuristics, but even so the macro may end up RefCell-ing
//!     immutable borrows, constants, or function pointers sometimes.
//!     You can help the macro by writing `(&mut var).method()` or
//!     `(&var).method()` instead of `var.method()`.
//!
//! ## Sample expansion
//! See [here](https://github.com/wishawa/enjoin/blob/main/tests/sample_expansion.rs).
pub use enjoin_macro::{join, join_auto_borrow};

pub mod polyfill {
    //! Polyfill for the rust Try (and related) trait that is currently unstable.
    //! See <https://doc.rust-lang.org/std/ops/trait.Try.html> for docs.
    //! Any divergences in behaviour should be considered bugs.
    //!
    //! Taken from [Iteritor](https://github.com/stormbrew/iteritor/blob/main/src/try_polyfill.rs).

    /*
    Copyright 2022 The Iteritor Authors
    Licensed under the Apache License, Version 2.0 (the "License");
    you may not use this file except in compliance with the License.
    You may obtain a copy of the License at
        https://www.apache.org/licenses/LICENSE-2.0
    Unless required by applicable law or agreed to in writing, software
    distributed under the License is distributed on an "AS IS" BASIS,
    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
    See the License for the specific language governing permissions and
    limitations under the License.
    */

    use core::{convert::Infallible, ops::ControlFlow};

    pub trait FromResidual<R = <Self as Try>::Residual> {
        fn from_residual(residual: R) -> Self;
    }

    pub trait Try: FromResidual<Self::Residual> {
        type Output;
        type Residual;

        fn from_output(output: Self::Output) -> Self;
        fn branch(self) -> ControlFlow<Self::Residual, Self::Output>;
    }

    impl<T> Try for Option<T> {
        type Output = T;
        type Residual = Option<Infallible>;

        fn from_output(output: Self::Output) -> Self {
            Some(output)
        }
        fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
            match self {
                Some(output) => ControlFlow::Continue(output),
                None => ControlFlow::Break(None),
            }
        }
    }

    impl<T, E> Try for Result<T, E> {
        type Output = T;
        type Residual = Result<Infallible, E>;

        fn from_output(output: Self::Output) -> Self {
            Ok(output)
        }
        fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
            match self {
                Ok(output) => ControlFlow::Continue(output),
                Err(err) => ControlFlow::Break(Err(err)),
            }
        }
    }

    impl<T> FromResidual for Option<T> {
        fn from_residual(_residual: Option<Infallible>) -> Self {
            None
        }
    }

    impl<T, E, F> FromResidual<Result<Infallible, E>> for Result<T, F>
    where
        F: From<E>,
    {
        #[track_caller]
        fn from_residual(residual: Result<Infallible, E>) -> Self {
            let err = F::from(residual.unwrap_err());
            Err(err)
        }
    }
}
