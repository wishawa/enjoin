use syn::{parse_quote, visit_mut::VisitMut, Expr, ExprBlock};

pub fn replace_trys(blocks: &mut Vec<ExprBlock>) {
    let mut replacer = TryReplacer { try_level: 0 };
    blocks.iter_mut().for_each(|block| {
        replacer.visit_expr_block_mut(block);
    });
}

struct TryReplacer {
    try_level: usize,
}
impl VisitMut for TryReplacer {
    fn visit_item_mut(&mut self, _i: &mut syn::Item) {}
    fn visit_expr_async_mut(&mut self, _i: &mut syn::ExprAsync) {}
    fn visit_expr_closure_mut(&mut self, _i: &mut syn::ExprClosure) {}

    fn visit_expr_try_block_mut(&mut self, i: &mut syn::ExprTryBlock) {
        self.try_level += 1;
        syn::visit_mut::visit_expr_try_block_mut(self, i);
        self.try_level -= 1;
    }
    fn visit_expr_mut(&mut self, i: &mut Expr) {
        syn::visit_mut::visit_expr_mut(self, i);
        if let Expr::Try(t) = i {
            if self.try_level == 0 {
                let e = &*t.expr;
                *i = parse_quote!((match ::enjoin::polyfill::Try::branch(#e) {
                    ::core::ops::ControlFlow::Break(b) => return ::enjoin::polyfill::FromResidual::from_residual(b),
                    ::core::ops::ControlFlow::Continue(c) => c
                }));
            }
        }
    }
}
