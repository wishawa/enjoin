#[pollster::test]
async fn sample_expansion() {
    async fn do_thing_a() -> i32 {
        20
    }
    async fn do_thing_b() -> Option<i32> {
        None
    }
    // This one is expanded (with Rust-Analyzer) from `inner_original` below.
    async fn inner() -> Option<&'static str> {
        let mut done = 0;
        'a: {
            #[allow(warnings)]
            {
                let __enjoin_borrows_cell = ::std::cell::RefCell::new((&mut done,));
                enum __enjoin_OutputEnum<__enjoin_Return, __enjoin_Keep> {
                    __enjoin_Keep(__enjoin_Keep),
                    __enjoin_Return(__enjoin_Return),
                    __enjoin_Break_a(()),
                }
                impl<__enjoin_Return, __enjoin_Keep> __enjoin_OutputEnum<__enjoin_Return, __enjoin_Keep> {
                    fn convert_breaking<__enjoin_TargetType>(
                        self,
                    ) -> ::core::ops::ControlFlow<
                        __enjoin_OutputEnum<__enjoin_Return, __enjoin_TargetType>,
                        __enjoin_Keep,
                    > {
                        match self {
                            Self::__enjoin_Keep(e) => ::core::ops::ControlFlow::Continue(e),
                            Self::__enjoin_Return(e) => ::core::ops::ControlFlow::Break(
                                __enjoin_OutputEnum::__enjoin_Return(e),
                            ),
                            Self::__enjoin_Break_a(_) => ::core::ops::ControlFlow::Break(
                                __enjoin_OutputEnum::__enjoin_Break_a(()),
                            ),
                        }
                    }
                }
                let mut __enjoin_pinned_futs = (
                    ::core::pin::pin!(async {
                        #[allow(unreachable_code)]
                        __enjoin_OutputEnum::__enjoin_Keep(
                            #[warn(unreachable_code)]
                            {
                                let mut __enjoin_borrows =
                                    ::std::cell::RefCell::borrow_mut(&__enjoin_borrows_cell);
                                let res = (
                                    (do_thing_a(), {
                                        ::core::mem::drop(__enjoin_borrows);
                                    })
                                        .0
                                        .await,
                                    {
                                        __enjoin_borrows = ::std::cell::RefCell::borrow_mut(
                                            &__enjoin_borrows_cell,
                                        );
                                    },
                                )
                                    .0;
                                if res > 3 {
                                    return __enjoin_OutputEnum::__enjoin_Return((Some("hello")));
                                } else {
                                    (*__enjoin_borrows.0) += 1;
                                    if (*__enjoin_borrows.0) == 2 {
                                        return __enjoin_OutputEnum::__enjoin_Break_a(());
                                    }
                                }
                            },
                        )
                    }),
                    ::core::pin::pin!(async {
                        #[allow(unreachable_code)]
                        __enjoin_OutputEnum::__enjoin_Keep(
                            #[warn(unreachable_code)]
                            {
                                let mut __enjoin_borrows =
                                    ::std::cell::RefCell::borrow_mut(&__enjoin_borrows_cell);
                                let _res = (match ::enjoin::polyfill::Try::branch(
                                    (
                                        (do_thing_b(), {
                                            ::core::mem::drop(__enjoin_borrows);
                                        })
                                            .0
                                            .await,
                                        {
                                            __enjoin_borrows = ::std::cell::RefCell::borrow_mut(
                                                &__enjoin_borrows_cell,
                                            );
                                        },
                                    )
                                        .0,
                                ) {
                                    ::core::ops::ControlFlow::Break(b) => {
                                        return __enjoin_OutputEnum::__enjoin_Return(
                                            (::enjoin::polyfill::FromResidual::from_residual(b)),
                                        )
                                    }
                                    ::core::ops::ControlFlow::Continue(c) => c,
                                });
                                (*__enjoin_borrows.0) += 1;
                            },
                        )
                    }),
                );
                let mut __enjoin_num_left = 2usize;
                let mut __enjoin_ouputs =
                    (::core::option::Option::None, ::core::option::Option::None);
                match ::core::future::poll_fn(|__enjoin_poll_cx| {
                    if ::core::option::Option::is_none(&__enjoin_ouputs.0) {
                        match ::core::future::Future::poll(
                            ::core::pin::Pin::as_mut(&mut __enjoin_pinned_futs.0),
                            __enjoin_poll_cx,
                        ) {
                            ::core::task::Poll::Ready(r) => {
                                match __enjoin_OutputEnum::convert_breaking(r) {
                                    ::core::ops::ControlFlow::Continue(v) => {
                                        __enjoin_num_left -= 1;
                                        __enjoin_ouputs.0 = ::core::option::Option::Some(v)
                                    }
                                    ::core::ops::ControlFlow::Break(b) => {
                                        return ::core::task::Poll::Ready(b)
                                    }
                                }
                            }
                            ::core::task::Poll::Pending => {}
                        }
                    }
                    if ::core::option::Option::is_none(&__enjoin_ouputs.1) {
                        match ::core::future::Future::poll(
                            ::core::pin::Pin::as_mut(&mut __enjoin_pinned_futs.1),
                            __enjoin_poll_cx,
                        ) {
                            ::core::task::Poll::Ready(r) => {
                                match __enjoin_OutputEnum::convert_breaking(r) {
                                    ::core::ops::ControlFlow::Continue(v) => {
                                        __enjoin_num_left -= 1;
                                        __enjoin_ouputs.1 = ::core::option::Option::Some(v)
                                    }
                                    ::core::ops::ControlFlow::Break(b) => {
                                        return ::core::task::Poll::Ready(b)
                                    }
                                }
                            }
                            ::core::task::Poll::Pending => {}
                        }
                    }
                    if __enjoin_num_left == 0 {
                        ::core::task::Poll::Ready(__enjoin_OutputEnum::__enjoin_Keep((
                            ::core::option::Option::unwrap(::core::option::Option::take(
                                &mut __enjoin_ouputs.0,
                            )),
                            ::core::option::Option::unwrap(::core::option::Option::take(
                                &mut __enjoin_ouputs.1,
                            )),
                        )))
                    } else {
                        ::core::task::Poll::Pending
                    }
                })
                .await
                {
                    __enjoin_OutputEnum::__enjoin_Keep(e) => e,
                    __enjoin_OutputEnum::__enjoin_Return(e) => return e,
                    __enjoin_OutputEnum::__enjoin_Break_a(_) => break 'a,
                }
            };
        };
        None
    }
    // This is the original code for the expanded code above.
    async fn inner_original() -> Option<&'static str> {
        let mut done = 0;
        'a: {
            enjoin::join_auto_borrow!(
                {
                    let res = do_thing_a().await;
                    if res > 3 {
                        return Some("hello");
                    } else {
                        done += 1;
                        if done == 2 {
                            break 'a;
                        }
                    }
                },
                {
                    let _res = do_thing_b().await?;
                    done += 1;
                }
            );
        };
        None
    }

    assert_eq!(inner().await, Some("hello"));
    assert_eq!(inner_original().await, Some("hello"));
}
