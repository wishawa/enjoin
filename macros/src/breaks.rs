use std::collections::HashMap;

use proc_macro2::Ident;
use quote::format_ident;
use syn::{parse_quote, visit_mut::VisitMut, Expr, ExprBlock, Lifetime};

#[derive(PartialEq, Eq, Hash)]
pub(crate) enum Escape {
    Break(Option<Lifetime>),
    Continue(Option<Lifetime>),
    Return,
}

impl Escape {
    pub fn variant_name(&self, private_ident: &Ident) -> Ident {
        match self {
            Escape::Break(Some(la)) => format_ident!("{}_Break_{}", private_ident, &la.ident),
            Escape::Break(None) => format_ident!("{}_Break", private_ident),
            Escape::Continue(Some(la)) => format_ident!("{}_Continue_{}", private_ident, &la.ident),
            Escape::Continue(None) => format_ident!("{}_Continue", private_ident),
            Escape::Return => format_ident!("{}_Return", private_ident),
        }
    }
}

pub(crate) struct BreakReplacer<'a> {
    pub output_type: &'a Ident,
    pub labels: Vec<Ident>,
    pub loop_level: usize,
    pub found: HashMap<Escape, Ident>,
    pub private_ident: &'a Ident,
}
macro_rules! visit_opt_label_block {
    ($self:ident, $visitor_name:ident, $i:ident, $inc_loop:literal) => {
        $self.loop_level += $inc_loop;
        if let Some(label) = &$i.label {
            $self.labels.push(label.name.ident.to_owned());
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
            Expr::Break(br) => (Escape::Break(br.label.to_owned()), br.expr.as_ref()),
            Expr::Continue(co) => (Escape::Continue(co.label.to_owned()), None),
            Expr::Return(re) => (Escape::Return, re.expr.as_ref()),
            _ => return,
        };
        let variant_name = match self.found.entry(esc) {
            std::collections::hash_map::Entry::Occupied(occ) => occ.into_mut(),
            std::collections::hash_map::Entry::Vacant(vac) => {
                let name = vac.key().variant_name(&self.private_ident);
                vac.insert(name)
            }
        };

        let out_type = self.output_type;
        let expr = expr.into_iter();
        *i = parse_quote!( return #out_type :: #variant_name ( #(#expr)* ) );
    }
}
