mod awaits;
mod breaks;
mod captures;
mod trys;

use std::{collections::HashMap, iter::repeat};

use breaks::{BreakReplacer, Escape};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse::Parse, parse_macro_input, parse_quote, punctuated::Punctuated, ExprBlock, Ident, Token,
};

/// Run given blocks of async code concurrently.
/// Use `break`/`continue`/`return`/`?` to jump out.
/// See the [crate documentation](https://docs.rs/enjoin/latest/enjoin/).
#[proc_macro]
pub fn join(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as MacroInput);
    match input.generate(false) {
        Ok(o) => o.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

/// Everything [join!] does,
/// plus the automatic shared mutable borrowing described in the
/// [crate documentation](https://docs.rs/enjoin/latest/enjoin/).
#[proc_macro]
pub fn join_auto_borrow(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as MacroInput);
    match input.generate(true) {
        Ok(o) => o.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

struct MacroInput(Vec<ExprBlock>);

impl Parse for MacroInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let blocks: Punctuated<ExprBlock, Token![,]> = Punctuated::parse_terminated(input)?;
        Ok(Self(blocks.into_iter().collect()))
    }
}

impl MacroInput {
    fn generate(self, make_borrows: bool) -> syn::Result<TokenStream> {
        let private_ident = Ident::new("__enjoin", Span::mixed_site());
        let borrows_tuple = format_ident!("{}_borrows", private_ident);
        let borrows_cell = format_ident!("{}_borrows_cell", private_ident);
        let Self(mut blocks) = self;
        let borrows = if make_borrows {
            captures::replace_captures_and_generate_borrows(
                &mut blocks,
                &borrows_tuple,
                &borrows_cell,
            )
        } else {
            None
        };

        let num = blocks.len();
        trys::desugar_trys(&mut blocks);

        let output_type = format_ident!("{}_OutputEnum", private_ident);
        let mut replacer = BreakReplacer {
            output_type: &output_type,
            labels: Vec::new(),
            loop_level: 0,
            found: HashMap::new(),
            private_ident: &private_ident,
        };
        blocks.iter_mut().for_each(|block| {
            use syn::visit_mut::VisitMut;
            replacer.visit_expr_block_mut(block);
        });

        let all_breaks = replacer
            .found
            .iter()
            .filter_map(|(escape, (ident, has_expr))| match escape {
                Escape::Break(_label) => Some((ident.to_owned(), has_expr)),
                _ => None,
            })
            .collect::<Vec<_>>();

        let br_variants_with_expr = all_breaks
            .iter()
            .filter_map(|(ident, has_expr)| if **has_expr { Some(ident) } else { None })
            .collect::<Vec<_>>();
        let br_variants_without_expr = all_breaks
            .iter()
            .filter_map(|(ident, has_expr)| if **has_expr { None } else { Some(ident) })
            .collect::<Vec<_>>();

        let co_variants = replacer
            .found
            .iter()
            .filter_map(|(escape, (ident, _))| match escape {
                Escape::Continue(_label) => Some(ident),
                _ => None,
            })
            .collect::<Vec<_>>();

        let re_variants = replacer
            .found
            .iter()
            .filter_map(|(escape, (ident, _))| match escape {
                Escape::Return => Some(ident),
                _ => None,
            })
            .collect::<Vec<_>>();

        let convert_breaking_ty = format_ident!("{}_TargetType", private_ident);
        let keep_ty = format_ident!("{}_Keep", private_ident);
        let return_type = quote!(
            enum #output_type <#(#br_variants_with_expr,)* #(#re_variants,)* #keep_ty> {
                #keep_ty (#keep_ty),
                #(#re_variants (#re_variants),)*
                #(#br_variants_with_expr (#br_variants_with_expr),)*
                #(#br_variants_without_expr (()) ,)*
                #(#co_variants (()),)*
            }
            impl <#(#br_variants_with_expr,)* #(#re_variants,)* #keep_ty> #output_type <#(#br_variants_with_expr,)* #(#re_variants,)* #keep_ty> {
                fn convert_breaking<#convert_breaking_ty>(self) -> ::core::ops::ControlFlow<#output_type <#(#br_variants_with_expr,)* #(#re_variants,)* #convert_breaking_ty>, #keep_ty> {
                    match self {
                        Self :: #keep_ty (e) => ::core::ops::ControlFlow::Continue (e),
                        #(Self :: #re_variants (e) => ::core::ops::ControlFlow::Break (#output_type :: #re_variants (e) ),)*
                        #(Self :: #br_variants_with_expr (e) => ::core::ops::ControlFlow::Break (#output_type :: #br_variants_with_expr (e) ),)*
                        #(Self :: #br_variants_without_expr (_) => ::core::ops::ControlFlow::Break (#output_type :: #br_variants_without_expr (()) ),)*
                        #(Self :: #co_variants (_) => ::core::ops::ControlFlow::Break (#output_type :: #co_variants (())) ,)*
                    }
                }
            }
        );

        let br_labels_with_expr =
            replacer
                .found
                .iter()
                .filter_map(|(escape, (_, has_expr))| match escape {
                    Escape::Break(label) if *has_expr => Some(label),
                    _ => None,
                });
        let br_labels_without_expr =
            replacer
                .found
                .iter()
                .filter_map(|(escape, (_, has_expr))| match escape {
                    Escape::Break(label) if !*has_expr => Some(label),
                    _ => None,
                });

        let co_labels = replacer
            .found
            .iter()
            .filter_map(|(escape, _ident)| match escape {
                Escape::Continue(label) => Some(label),
                _ => None,
            });

        if borrows.is_some() {
            awaits::replace_awaits(&mut blocks, &borrows_tuple, &borrows_cell);
        }

        let poll_cx = format_ident!("{}_poll_cx", private_ident);
        let pinned_futs = format_ident!("{}_pinned_futs", private_ident);
        let indices = (0..num).map(syn::Index::from).collect::<Vec<_>>();
        let num_left = format_ident!("{}_num_left", private_ident);
        let outputs = format_ident!("{}_ouputs", private_ident);
        let poller = quote! (
            ::core::future::poll_fn(|#poll_cx| {
                #(
                    if ::core::option::Option::is_none(& #outputs . #indices) {
                        match ::core::future::Future::poll(::core::pin::Pin::as_mut(&mut #pinned_futs . #indices), #poll_cx) {
                            ::core::task::Poll::Ready (r) => match #output_type :: convert_breaking (r) {
                                ::core::ops::ControlFlow::Continue (v) => {
                                    #num_left -= 1;
                                    #outputs . #indices = ::core::option::Option::Some(v)
                                },
                                ::core::ops::ControlFlow::Break (b) => return ::core::task::Poll::Ready (b),
                            },
                            ::core::task::Poll::Pending => {}
                        }
                    }
                )*
                if #num_left == 0 {
                    ::core::task::Poll::Ready (
                        #output_type :: #keep_ty (
                            (#(::core::option::Option::unwrap(::core::option::Option::take(&mut #outputs . #indices)),)*)
                        )
                    )
                }
                else {
                    ::core::task::Poll::Pending
                }
            })
        );
        let none: syn::Path = parse_quote!(::core::option::Option::None);
        let nones = repeat(&none).take(num);
        Ok(quote! {
            {
                #borrows
                #return_type
                let mut #pinned_futs = (
                    #(::core::pin::pin!(async {
                        #[allow(unreachable_code)]
                        #output_type :: #keep_ty (
                            #[warn(unreachable_code)]
                            #blocks
                        )
                    }),)*
                );
                let mut #num_left = #num;
                let mut #outputs = (#(#nones,)*);
                match #poller .await {
                    #output_type :: #keep_ty (e) => e,
                    #(#output_type :: #re_variants (e) => return e,)*
                    #(#output_type :: #br_variants_with_expr (e) => break #br_labels_with_expr e,)*
                    #(#output_type :: #br_variants_without_expr (_) => break #br_labels_without_expr,)*
                    #(#output_type :: #co_variants (_) => continue #co_labels,)*
                }
            }
        })
    }
}
