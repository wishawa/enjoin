pub use enjoin_macro::join;

pub mod for_macro_only {
    pub use futures;
}

pub mod polyfill {
    //! Polyfill for the rust Try (and related) trait that is currently unstable.
    //! See https://doc.rust-lang.org/std/ops/trait.Try.html for docs.
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
