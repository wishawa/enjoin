mod awaits;
mod captures;
// mod breaks;
// mod trys;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{parse::Parse, parse_macro_input, punctuated::Punctuated, ExprBlock, Ident, Token};

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
        awaits::replace_awaits(&mut blocks, &borrows_tuple, &borrows_cell);
        Ok(quote! {
            #borrows
            ::enjoin::for_macro_only::futures::join! {
                #(async #blocks ,)*
            }
        })
    }
}
