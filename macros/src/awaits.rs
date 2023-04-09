use proc_macro2::Ident;
use syn::{parse_quote, visit_mut::VisitMut, Expr, ExprBlock};

pub fn replace_awaits(
    blocks: &mut Vec<ExprBlock>,
    borrows_tuple_name: &Ident,
    borrows_cell_name: &Ident,
) {
    let mut replacer = AwaitReplacer {
        borrows_tuple_name,
        borrows_cell_name,
    };
    blocks.iter_mut().for_each(|block| {
        replacer.visit_expr_block_mut(block);
        block.block.stmts.insert(
            0,
            parse_quote!(let mut #borrows_tuple_name = ::std::cell::RefCell::borrow_mut(&#borrows_cell_name);),
        );
    });
}
struct AwaitReplacer<'a> {
    borrows_tuple_name: &'a Ident,
    borrows_cell_name: &'a Ident,
}
impl<'a> VisitMut for AwaitReplacer<'a> {
    fn visit_item_mut(&mut self, _i: &mut syn::Item) {}
    fn visit_expr_async_mut(&mut self, _i: &mut syn::ExprAsync) {}
    fn visit_expr_closure_mut(&mut self, _i: &mut syn::ExprClosure) {}

    fn visit_expr_mut(&mut self, i: &mut Expr) {
        if let Expr::Await(aw) = i {
            let borrows_name = self.borrows_tuple_name;
            let borrows_cell_name = self.borrows_cell_name;
            let base = &aw.base;
            *i = parse_quote!(
                (
                    (
                        #base,
                        {::core::mem::drop( #borrows_name );},
                    ).0.await,
                    {#borrows_name = ::std::cell::RefCell::borrow_mut(&#borrows_cell_name);}
                ).0
            );
        } else {
            syn::visit_mut::visit_expr_mut(self, i);
        }
    }
}
