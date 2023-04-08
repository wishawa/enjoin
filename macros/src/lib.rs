mod awaits;
mod breaks;
mod captures;
mod trys;

use std::{collections::HashMap, iter::repeat};

use breaks::{BreakReplacer, Escape};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse::Parse, parse_macro_input, parse_quote, punctuated::Punctuated, ExprBlock, Ident,
    Lifetime, Token,
};

#[proc_macro]
pub fn join(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as MacroInput);
    match input.generate() {
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
    fn generate(self) -> syn::Result<TokenStream> {
        let private_ident = Ident::new("__enjoin", Span::mixed_site());
        let borrows_tuple = format_ident!("{}_borrows", private_ident);
        let borrows_cell = format_ident!("{}_borrows_cell", private_ident);
        let Self(mut blocks) = self;
        let borrows = captures::replace_captures_and_generate_borrows(
            &mut blocks,
            &borrows_tuple,
            &borrows_cell,
        );

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
            .filter_map(|(escape, ident)| match escape {
                Escape::Break(_label) => Some(ident.to_owned()),
                _ => None,
            })
            .collect::<Vec<_>>();

        let br_generics = all_breaks.iter().collect::<Vec<_>>();
        let br_variants = &br_generics;
        let co_variants = replacer
            .found
            .iter()
            .filter_map(|(escape, ident)| match escape {
                Escape::Continue(_label) => Some(ident),
                _ => None,
            })
            .collect::<Vec<_>>();

        let re_generics = replacer
            .found
            .iter()
            .filter_map(|(escape, ident)| match escape {
                Escape::Return => Some(ident),
                _ => None,
            })
            .collect::<Vec<_>>();
        let re_variants = &re_generics;

        let convert_breaking_ty = format_ident!("{}_TargetType", private_ident);
        let keep_ty = format_ident!("{}_Keep", private_ident);
        let return_type = quote!(
            enum #output_type <#(#br_generics,)* #(#re_generics,)* #keep_ty> {
                #keep_ty (#keep_ty),
                #(#re_variants (#re_generics),)*
                #(#br_variants (#br_generics),)*
                #(#co_variants,)*
            }
            impl <#(#br_generics,)* #(#re_generics,)* #keep_ty> #output_type <#(#br_generics,)* #(#re_generics,)* #keep_ty> {
                fn convert_breaking<#convert_breaking_ty>(self) -> ::core::ops::ControlFlow<#output_type <#(#br_generics,)* #(#re_generics,)* #convert_breaking_ty>, #keep_ty> {
                    match self {
                        Self :: #keep_ty (e) => ::core::ops::ControlFlow::Continue (e),
                        #(Self :: #re_variants (e) => ::core::ops::ControlFlow::Break (#output_type :: #re_variants (e) ),)*
                        #(Self :: #br_variants (e) => ::core::ops::ControlFlow::Break (#output_type :: #br_variants (e) ),)*
                        #(Self :: #co_variants => ::core::ops::ControlFlow::Break (#output_type :: #co_variants) ,)*
                    }
                }
            }
        );
        let br_lifetimes = replacer
            .found
            .iter()
            .filter_map(|(escape, _ident)| match escape {
                Escape::Break(label) => {
                    Some(Lifetime::new(&format!("'{}", label), Span::call_site()))
                }
                _ => None,
            });
        let co_lifetimes = replacer
            .found
            .iter()
            .filter_map(|(escape, _ident)| match escape {
                Escape::Continue(label) => {
                    Some(Lifetime::new(&format!("'{}", label), Span::call_site()))
                }
                _ => None,
            });

        awaits::replace_awaits(&mut blocks, &borrows_tuple, &borrows_cell);

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
                    #(::core::pin::pin!(async { #output_type :: #keep_ty (#blocks) }),)*
                );
                let mut #num_left = #num;
                let mut #outputs = (#(#nones,)*);
                match #poller .await {
                    #output_type :: #keep_ty (e) => e,
                    #(#output_type :: #re_variants (e) => return e,)*
                    #(#output_type :: #br_variants (e) => break #br_lifetimes e,)*
                    #(#output_type :: #co_variants => continue #co_lifetimes,)*
                }
            }
        })
    }
}
