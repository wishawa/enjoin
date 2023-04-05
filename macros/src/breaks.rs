use std::collections::HashMap;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::{parse_quote, visit_mut::VisitMut, Expr, ExprBlock, Block};

fn replace_escapes_and_generate_(blocks: &mut Vec<ExprBlock>) -> TokenStream {
    let output_type = Ident::new("SJoin_JoinOutput", Span::mixed_site());
    let mut replacer = BreakReplacer {
        output_type: &output_type,
        labels: Vec::new(),
        loop_level: 0,
        found: HashMap::new(),
    };
    blocks.iter_mut().for_each(|block| {
        replacer.visit_expr_block_mut(block);
    });

    let all_breaks = replacer
        .found
        .iter()
        .filter_map(|(escape, ident)| match escape {
            Escape::Break(_label) => Some(ident),
            _ => None,
        })
        .collect::<Vec<_>>();

    let br_variants = all_breaks.iter();
    let br_generics = all_breaks.iter();
    let br_generics_cpy = all_breaks.iter();
    let co_variants = replacer
        .found
        .iter()
        .filter_map(|(escape, ident)| match escape {
            Escape::Continue(_label) => Some(ident),
            _ => None,
        });
    quote!(
        enum #output_type <#(#br_generics_cpy,)* Return, Regular> {
            Regular(Regular),
			Return(Return),
            #(#br_variants (#br_generics),)*
            #(#co_variants,)*
        }
		
    )
}

#[derive(PartialEq, Eq, Hash)]
enum Escape {
    Break(String),
    Continue(String),
}
impl Escape {
    fn variant_name(&self) -> Ident {
        let (ty, la) = match self {
            Escape::Break(la) => ("Break", &**la),
            Escape::Continue(la) => ("Continue", &**la),
        };
        format_ident!("{}_{}", ty, la)
    }
}

struct BreakReplacer<'a> {
    output_type: &'a Ident,
    labels: Vec<String>,
    loop_level: usize,
    found: HashMap<Escape, Ident>,
}
macro_rules! visit_opt_label_block {
    ($self:ident, $visitor_name:ident, $i:ident, $inc_loop:literal) => {
        $self.loop_level += $inc_loop;
        if let Some(label) = &$i.label {
            $self.labels.push(label.name.ident.to_string());
            syn::visit_mut::$visitor_name($self, $i);
            $self.labels.pop();
        } else {
            syn::visit_mut::$visitor_name($self, $i);
        }
        $self.loop_level -= $inc_loop;
    };
}
impl<'a> VisitMut for BreakReplacer<'a> {
    fn visit_item_mut(&mut self, _i: &mut syn::Item) {}
    fn visit_expr_async_mut(&mut self, _i: &mut syn::ExprAsync) {}
    fn visit_expr_closure_mut(&mut self, _i: &mut syn::ExprClosure) {}

    fn visit_expr_block_mut(&mut self, i: &mut ExprBlock) {
        visit_opt_label_block!(self, visit_expr_block_mut, i, 0);
    }
    fn visit_expr_for_loop_mut(&mut self, i: &mut syn::ExprForLoop) {
        visit_opt_label_block!(self, visit_expr_for_loop_mut, i, 1);
    }
    fn visit_expr_while_mut(&mut self, i: &mut syn::ExprWhile) {
        visit_opt_label_block!(self, visit_expr_while_mut, i, 1);
    }
    fn visit_expr_loop_mut(&mut self, i: &mut syn::ExprLoop) {
        visit_opt_label_block!(self, visit_expr_loop_mut, i, 1);
    }

    fn visit_expr_mut(&mut self, i: &mut Expr) {
        syn::visit_mut::visit_expr_mut(self, i);
        let (esc, expr) = match i {
            Expr::Break(br) => (
                Escape::Break(
                    br.label
                        .as_ref()
                        .map(|l| l.ident.to_string())
                        .unwrap_or_default(),
                ),
                br.expr.as_ref(),
            ),
            Expr::Continue(co) => (
                Escape::Continue(
                    co.label
                        .as_ref()
                        .map(|l| l.ident.to_string())
                        .unwrap_or_default(),
                ),
                None,
            ),
            _ => return,
        };
        let variant_name = match self.found.entry(esc) {
            std::collections::hash_map::Entry::Occupied(occ) => occ.into_mut(),
            std::collections::hash_map::Entry::Vacant(vac) => {
                let name = vac.key().variant_name();
                vac.insert(name)
            }
        };

        let out_type = self.output_type;
        let expr = expr.into_iter();
        *i = parse_quote!( return #out_type :: #variant_name (( #(#expr)* )) );
    }
    fn visit_expr_return_mut(&mut self, i: &mut syn::ExprReturn) {
        syn::visit_mut::visit_expr_return_mut(self, i);
        let out_type = self.output_type;
        *i = if let Some(expr) = i.expr.as_deref() {
            parse_quote!( return #out_type :: Return ( #expr ) )
        } else {
            parse_quote!( return #out_type :: Return (()) )
        };
    }
}
