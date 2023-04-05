use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{parse_quote, visit::Visit, visit_mut::VisitMut, Expr, ExprBlock};

pub fn replace_captures_and_generate_borrows(
    blocks: &mut Vec<ExprBlock>,
    borrows_tuple_name: &Ident,
    borrows_cell_name: &Ident,
) -> TokenStream {
    // Visit all the blocks we want to join, figuring out which captures what.
    let mut block_captures = blocks
        .iter_mut()
        .map(|block| {
            let mut captures = HashMap::<CaptureName, CaptureMutability>::new();
            let mut collector = CaptureVisitor {
                locals: Locals::default(),
                do_on_expr: |i: &mut Expr, locals: &mut Locals| -> bool {
                    use CaptureMutability::*;
                    if let Some(capt) = get_capture(i, locals) {
                        match captures.entry(capt.name) {
                            std::collections::hash_map::Entry::Occupied(mut occ) => {
                                *occ.get_mut() = occ.get().merge(capt.mutability);
                            }
                            std::collections::hash_map::Entry::Vacant(vac) => {
                                vac.insert(match capt.mutability {
                                    Immutable => Immutable,
                                    _ => Unknown,
                                });
                            }
                        }
                        true
                    } else {
                        false
                    }
                },
            };
            collector.visit_expr_block_mut(block);
            captures
        })
        .collect::<Vec<_>>();

    // Gather all the captured variables together.
    let mut all_captures = block_captures.iter().fold(
        HashMap::new(),
        |mut h: HashMap<CaptureName, (CaptureMutability, Vec<usize>)>, item| {
            item.iter()
                .enumerate()
                .for_each(|(block_num, (name, mutability))| {
                    if let Some((ex_mut, ex_nums)) = h.get_mut(name) {
                        ex_nums.push(block_num);
                        *ex_mut = ex_mut.merge(*mutability);
                    } else {
                        h.insert(name.to_owned(), (*mutability, vec![block_num]));
                    }
                });
            h
        },
    );
    // Filter for ones that are captured by at least 2 blocks and not known to be immutable.
    // These are the ones we need to wrap in cells.
    all_captures.retain(|_, (mutability, nums)| {
        *mutability == CaptureMutability::Unknown && nums.len() > 1
    });

    // Remove the captures that we don't need to wrap.
    block_captures
        .iter_mut()
        .for_each(|block| block.retain(|k, _| all_captures.contains_key(k)));

    let mut names = all_captures.keys().collect::<Vec<_>>();
    names.sort();
    for sp in (1..names.len()).rev() {
        let (anc, des) = names.split_at(sp);
        let anc = anc.last().unwrap();
        let des = des.first().unwrap();
        if des.ident == anc.ident && des.members.starts_with(&anc.members) {
            names.swap_remove(sp);
        }
    }
    let replacements = names
        .iter()
        .enumerate()
        .map(|(idx, &name)| (name, idx as u32))
        .collect();
    let mut capture_replacer = CaptureVisitor {
        locals: Locals::default(),
        do_on_expr: |i: &mut Expr, locals: &mut Locals| -> bool {
            replace_capture(i, locals, &replacements, &borrows_tuple_name)
        },
    };
    blocks.iter_mut().for_each(|block| {
        capture_replacer.visit_expr_block_mut(block);
    });

    let borrows = names.iter().map(|name| &name.expr);

    quote!(
        let #borrows_cell_name = ::std::cell::RefCell::new((
            #(&mut #borrows ,)*
        ));
    )
}

#[derive(Clone)]
struct CaptureName {
    ident: String,
    members: Vec<String>,
    expr: Option<Expr>,
}
impl PartialEq for CaptureName {
    fn eq(&self, other: &Self) -> bool {
        self.ident == other.ident && self.members == other.members
    }
}
impl Eq for CaptureName {}
impl Hash for CaptureName {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ident.hash(state);
        self.members.hash(state);
    }
}
impl PartialOrd for CaptureName {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.ident.partial_cmp(&other.ident) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.members.partial_cmp(&other.members)
    }
}
impl Ord for CaptureName {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}

fn member_to_string(m: &syn::Member) -> String {
    match m {
        syn::Member::Named(n) => n.to_string(),
        syn::Member::Unnamed(u) => u.index.to_string(),
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum CaptureMutability {
    Unknown,
    Immutable,
    Mutable,
}
impl CaptureMutability {
    fn merge(self, other: Self) -> Self {
        use CaptureMutability::*;
        match (self, other) {
            (Immutable, Immutable) => Immutable,
            _ => Unknown,
        }
    }
}
#[derive(Clone, PartialEq, Eq)]
struct Capture {
    name: CaptureName,
    mutability: CaptureMutability,
}

fn get_capture(i: &Expr, locals: &Locals) -> Option<Capture> {
    fn get_capture_inner<'i>(i: &'i syn::Expr, locals: &Locals) -> Option<CaptureName> {
        match i {
            Expr::Path(p) if p.path.segments.len() == 1 => {
                if let Some(syn::PathSegment {
                    arguments: syn::PathArguments::None,
                    ident,
                }) = p.path.segments.first()
                {
                    let s = ident.to_string();
                    (!s.chars().next().unwrap().is_ascii_uppercase() && !locals.contains(&s)).then(
                        || CaptureName {
                            ident: ident.to_string(),
                            members: Vec::new(),
                            expr: None,
                        },
                    )
                } else {
                    None
                }
            }
            Expr::Field(f) => get_capture_inner(&f.base, locals).map(|mut c| {
                c.members.push(member_to_string(&f.member));
                c
            }),
            _ => None,
        }
    }
    match i {
        Expr::Path(_) | Expr::Field(_) | Expr::Reference(_) => {
            let mut capt = get_capture_inner(i, locals)?;
            capt.expr = Some(i.clone());
            let mut out = Capture {
                name: capt,
                mutability: CaptureMutability::Unknown,
            };
            match i {
                Expr::Reference(r) => {
                    out.mutability = if r.mutability.is_some() {
                        CaptureMutability::Mutable
                    } else {
                        CaptureMutability::Immutable
                    };
                }
                _ => {}
            }
            Some(out)
        }
        _ => None,
    }
}

fn replace_capture(
    i: &mut Expr,
    locals: &Locals,
    replacements: &HashMap<&CaptureName, u32>,
    borrows_tuple_name: &Ident,
) -> bool {
    enum ReplaceResult {
        NotFound,
        Replacing(CaptureName),
        Done,
    }

    fn replace_capture_inner(
        i: &mut Expr,
        locals: &Locals,
        replacements: &HashMap<&CaptureName, u32>,
        borrows_tuple_name: &Ident,
    ) -> ReplaceResult {
        let capt = match i {
            Expr::Path(p) if p.path.segments.len() == 1 => {
                if let Some(syn::PathSegment {
                    arguments: syn::PathArguments::None,
                    ident,
                }) = p.path.segments.first()
                {
                    let s = ident.to_string();
                    if !s.chars().next().unwrap().is_ascii_uppercase() && !locals.contains(&s) {
                        CaptureName {
                            ident: ident.to_string(),
                            members: Vec::new(),
                            expr: None,
                        }
                    } else {
                        return ReplaceResult::NotFound;
                    }
                } else {
                    return ReplaceResult::NotFound;
                }
            }
            Expr::Field(f) => {
                match replace_capture_inner(&mut *f.base, locals, replacements, borrows_tuple_name)
                {
                    ReplaceResult::Replacing(mut c) => {
                        c.members.push(member_to_string(&f.member));
                        c
                    }
                    x => return x,
                }
            }
            _ => return ReplaceResult::NotFound,
        };
        if let Some(&member) = replacements.get(&capt) {
            let member = syn::Member::Unnamed(syn::Index {
                index: member,
                span: Span::mixed_site(),
            });
            *i = parse_quote!(( * #borrows_tuple_name . #member ));
            ReplaceResult::Done
        } else {
            ReplaceResult::Replacing(capt)
        }
    }
    match replace_capture_inner(i, locals, replacements, borrows_tuple_name) {
        ReplaceResult::Done => true,
        _ => false,
    }
}

#[derive(Default)]
struct Locals {
    all: HashSet<String>,
    stack: Vec<Vec<String>>,
}
impl Locals {
    fn add(&mut self, pat: &syn::Pat) {
        self.visit_pat(pat);
    }
    fn push_stack(&mut self) {
        self.stack.push(Vec::new());
    }
    fn pop_stack(&mut self) {
        for ident in self.stack.pop().unwrap() {
            self.all.remove(&ident);
        }
    }
    fn contains(&self, ident: &str) -> bool {
        self.all.contains(ident)
    }
}
impl<'ast> Visit<'ast> for Locals {
    fn visit_pat_ident(&mut self, i: &'ast syn::PatIdent) {
        if self.all.insert(i.ident.to_string()) {
            self.stack.last_mut().unwrap().push(i.ident.to_string());
        }
    }
}

/// A syn Visitor for finding all the variables that an async block captures.
/// It needs to be aware of which variables are local to the async block,
/// and which are captured from outside.
struct CaptureVisitor<F> {
    locals: Locals,
    do_on_expr: F,
}

impl<F: FnMut(&mut Expr, &mut Locals) -> bool> VisitMut for CaptureVisitor<F> {
    // Items cannot capture anything.
    fn visit_item_mut(&mut self, _i: &mut syn::Item) {}

    // These expressions create new bindings.
    fn visit_expr_if_mut(&mut self, i: &mut syn::ExprIf) {
        if let Expr::Let(el) = &mut *i.cond {
            self.visit_expr_mut(&mut el.expr);
            self.locals.push_stack();
            self.locals.add(&el.pat);
            self.visit_block_mut(&mut i.then_branch);
            self.locals.pop_stack();
            if let Some((_, eb)) = &mut i.else_branch {
                self.visit_expr_mut(&mut *eb);
            }
        } else {
            syn::visit_mut::visit_expr_if_mut(self, i);
        }
    }
    fn visit_expr_while_mut(&mut self, i: &mut syn::ExprWhile) {
        if let Expr::Let(el) = &mut *i.cond {
            self.visit_expr_mut(&mut el.expr);
            self.locals.push_stack();
            self.locals.add(&el.pat);
            self.visit_block_mut(&mut i.body);
            self.locals.pop_stack();
        } else {
            syn::visit_mut::visit_expr_while_mut(self, i);
        }
    }
    fn visit_expr_for_loop_mut(&mut self, i: &mut syn::ExprForLoop) {
        self.visit_expr_mut(&mut i.expr);
        self.locals.push_stack();
        self.locals.add(&*i.pat);
        self.visit_block_mut(&mut i.body);
        self.locals.pop_stack();
    }
    fn visit_arm_mut(&mut self, i: &mut syn::Arm) {
        self.locals.push_stack();
        self.locals.add(&i.pat);
        if let Some((_, guard)) = &mut i.guard {
            syn::visit_mut::visit_expr_mut(self, &mut *guard);
        }
        syn::visit_mut::visit_expr_mut(self, &mut i.body);
        self.locals.pop_stack();
    }
    fn visit_expr_closure_mut(&mut self, i: &mut syn::ExprClosure) {
        self.locals.push_stack();
        i.inputs.iter().for_each(|arg| self.locals.add(arg));
        self.visit_expr_mut(&mut *i.body);
        self.locals.pop_stack();
    }
    fn visit_local_mut(&mut self, i: &mut syn::Local) {
        syn::visit_mut::visit_local_mut(self, i);
        self.locals.add(&i.pat);
    }

    fn visit_block_mut(&mut self, i: &mut syn::Block) {
        self.locals.push_stack();
        syn::visit_mut::visit_block_mut(self, i);
        self.locals.pop_stack();
    }

    // Find variable expressions that we need to modify.
    fn visit_expr_mut(&mut self, i: &mut Expr) {
        if !(&mut self.do_on_expr)(i, &mut self.locals) {
            syn::visit_mut::visit_expr_mut(self, i);
        }
    }
}
