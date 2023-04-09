use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    hash::Hash,
};

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{parse_quote, visit::Visit, visit_mut::VisitMut, Expr, ExprBlock, Member};

pub fn replace_captures_and_generate_borrows(
    blocks: &mut Vec<ExprBlock>,
    borrows_tuple_name: &Ident,
    borrows_cell_name: &Ident,
) -> Option<TokenStream> {
    // Visit all the blocks we want to join, figuring out which captures what.
    let block_captures = blocks
        .iter_mut()
        .map(|block| {
            let mut collector = CaptureFinder {
                locals: Locals::default(),
                found: HashMap::new(),
            };
            collector.visit_expr_block(block);
            collector.found
        })
        .collect::<Vec<_>>();

    // Gather all the captured variables together.
    let mut all_captures = block_captures.into_iter().fold(
        HashMap::new(),
        |mut h: HashMap<Capture, (bool, Vec<&syn::Expr>)>, item| {
            item.into_iter()
                .for_each(|(name, (immutable, locations))| match h.entry(name) {
                    std::collections::hash_map::Entry::Occupied(mut occ) => {
                        occ.get_mut().0 &= immutable;
                        occ.get_mut().1.extend(locations);
                    }
                    std::collections::hash_map::Entry::Vacant(vac) => {
                        vac.insert((immutable, locations));
                    }
                });
            h
        },
    );

    // Filter for ones that are captured by at least 2 blocks and not known to be immutable.
    // These are the ones we need to wrap in cells.
    all_captures.retain(|_, (immutable, all_locations)| !*immutable && all_locations.len() > 1);

    let mut names = Vec::new();
    let mut all_captures = all_captures
        .iter_mut()
        .map(|(capt, (_imm, locs))| {
            names.push(capt);
            (capt, core::mem::take(locs))
        })
        .collect::<HashMap<_, _>>();

    names.sort();
    let mut temp_buf = Vec::new();
    for sp in (1..names.len()).rev() {
        let (anc, des) = names.split_at(sp);
        let anc = anc.last().unwrap();
        let dsc = des.first().unwrap();
        if dsc.root == anc.root && dsc.members.starts_with(&anc.members) {
            let depth_dif = dsc.members.len() - anc.members.len();
            temp_buf.extend(
                all_captures
                    .remove(*dsc)
                    .unwrap()
                    .into_iter()
                    .map(|ex| access_field(ex, depth_dif)),
            );
            all_captures
                .get_mut(*anc)
                .unwrap()
                .extend(temp_buf.drain(..));
            names.swap_remove(sp);
        }
    }

    let all_captures = all_captures.into_iter().collect::<Vec<_>>();

    let borrows = all_captures
        .iter()
        .map(|(Capture { root, members }, _locs)| {
            let members = members.iter().map(|m| &m.member);
            let b: syn::Expr = parse_quote!( #root #( . #members )* );
            b
        });

    let replacements = all_captures
        .iter()
        .enumerate()
        .flat_map(|(idx, (_capt, locs))| {
            let index = syn::Index::from(idx);
            locs.iter().map(move |&loc| {
                (
                    loc as *const syn::Expr,
                    parse_quote!( (* #borrows_tuple_name . #index) ),
                )
            })
        })
        .collect::<HashMap<_, _>>();

    let mut replacer = CaptureReplacer { replacements };

    if !names.is_empty() {
        let out = quote!(
            let #borrows_cell_name = ::std::cell::RefCell::new((
                #(&mut #borrows ,)*
            ));
        );
        blocks.iter_mut().for_each(|block| {
            replacer.visit_expr_block_mut(block);
        });
        Some(out)
    } else {
        None
    }
}

fn access_field(ex: &syn::Expr, depth: usize) -> &syn::Expr {
    if depth == 0 {
        ex
    } else {
        match ex {
            Expr::Field(f) => access_field(&*f.base, depth - 1),
            _ => panic!("no field to access"),
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Capture {
    root: Ident,
    members: Vec<CaptureMember>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct CaptureMember {
    member: Member,
}

impl PartialOrd for CaptureMember {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (&self.member, &other.member) {
            (Member::Named(this), Member::Named(other)) => this.partial_cmp(other),
            (Member::Named(_), Member::Unnamed(_)) => Some(Ordering::Greater),
            (Member::Unnamed(_), Member::Named(_)) => Some(Ordering::Less),
            (Member::Unnamed(this), Member::Unnamed(other)) => this.index.partial_cmp(&other.index),
        }
    }
}
impl Ord for CaptureMember {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

fn get_capture_field(i: &syn::Expr, locals: &Locals) -> Option<Capture> {
    match i {
        Expr::Path(p) if p.path.segments.len() == 1 => {
            if let Some(syn::PathSegment {
                arguments: syn::PathArguments::None,
                ident,
            }) = p.path.segments.first()
            {
                let s = ident.to_string();
                (!s.chars().next().unwrap().is_ascii_uppercase() && !locals.contains(&ident)).then(
                    || Capture {
                        root: ident.to_owned(),
                        members: Vec::new(),
                    },
                )
            } else {
                None
            }
        }
        Expr::Field(f) => get_capture_field(i, locals).map(|mut c| {
            c.members.push(CaptureMember {
                member: f.member.to_owned(),
            });
            c
        }),
        _ => None,
    }
}

#[derive(Default)]
struct Locals {
    all: HashSet<Ident>,
    stack: Vec<Vec<Ident>>,
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
    fn contains(&self, ident: &Ident) -> bool {
        self.all.contains(ident)
    }
}
impl<'ast> Visit<'ast> for Locals {
    fn visit_pat_ident(&mut self, i: &'ast syn::PatIdent) {
        if self.all.insert(i.ident.to_owned()) {
            self.stack.last_mut().unwrap().push(i.ident.to_owned());
        }
    }
}

/// A syn Visitor for finding all the variables that an async block captures.
/// It needs to be aware of which variables are local to the async block,
/// and which are captured from outside.
struct CaptureFinder<'ast> {
    locals: Locals,
    found: HashMap<Capture, (bool, Vec<&'ast syn::Expr>)>,
}

impl<'ast> Visit<'ast> for CaptureFinder<'ast> {
    // Items cannot capture anything.
    fn visit_item(&mut self, _i: &'ast syn::Item) {}

    // These expressions create new bindings.
    fn visit_expr_if(&mut self, i: &'ast syn::ExprIf) {
        if let Expr::Let(el) = &*i.cond {
            self.visit_expr(&el.expr);
            self.locals.push_stack();
            self.locals.add(&el.pat);
            self.visit_block(&i.then_branch);
            self.locals.pop_stack();
            if let Some((_, eb)) = &i.else_branch {
                self.visit_expr(&*eb);
            }
        } else {
            syn::visit::visit_expr_if(self, i);
        }
    }
    fn visit_expr_while(&mut self, i: &'ast syn::ExprWhile) {
        if let Expr::Let(el) = &*i.cond {
            self.visit_expr(&el.expr);
            self.locals.push_stack();
            self.locals.add(&el.pat);
            self.visit_block(&i.body);
            self.locals.pop_stack();
        } else {
            syn::visit::visit_expr_while(self, i);
        }
    }
    fn visit_expr_for_loop(&mut self, i: &'ast syn::ExprForLoop) {
        self.visit_expr(&i.expr);
        self.locals.push_stack();
        self.locals.add(&*i.pat);
        self.visit_block(&i.body);
        self.locals.pop_stack();
    }
    fn visit_arm(&mut self, i: &'ast syn::Arm) {
        self.locals.push_stack();
        self.locals.add(&i.pat);
        if let Some((_, guard)) = &i.guard {
            syn::visit::visit_expr(self, &*guard);
        }
        syn::visit::visit_expr(self, &i.body);
        self.locals.pop_stack();
    }
    fn visit_expr_closure(&mut self, i: &'ast syn::ExprClosure) {
        self.locals.push_stack();
        i.inputs.iter().for_each(|arg| self.locals.add(arg));
        self.visit_expr(&*i.body);
        self.locals.pop_stack();
    }
    fn visit_local(&mut self, i: &'ast syn::Local) {
        syn::visit::visit_local(self, i);
        self.locals.add(&i.pat);
    }

    fn visit_block(&mut self, i: &'ast syn::Block) {
        self.locals.push_stack();
        syn::visit::visit_block(self, i);
        self.locals.pop_stack();
    }

    // Find variable expressions that we need to modify.
    fn visit_expr(&mut self, i: &'ast Expr) {
        let (ex, immut) = match i {
            Expr::Path(_) | Expr::Field(_) => (i, false),
            Expr::Reference(r) => (&*r.expr, r.mutability.is_none()),
            Expr::Call(c) => {
                match &*c.func {
                    Expr::Path(p) if p.path.segments.len() == 1 => {}
                    _ => {
                        syn::visit::visit_expr(self, &*c.func);
                    }
                }
                c.args.iter().for_each(|arg| {
                    syn::visit::visit_expr(self, arg);
                });
                return;
            }
            _ => {
                syn::visit::visit_expr(self, i);
                return;
            }
        };
        if let Some(mut capt) = get_capture_field(ex, &self.locals) {
            capt.members.reverse();
            match self.found.entry(capt) {
                std::collections::hash_map::Entry::Occupied(mut occ) => {
                    occ.get_mut().0 &= immut;
                    occ.get_mut().1.push(ex as _);
                }
                std::collections::hash_map::Entry::Vacant(vac) => {
                    vac.insert((immut, vec![ex as _]));
                }
            }
        } else {
            syn::visit::visit_expr(self, i);
        }
    }
}

struct CaptureReplacer {
    replacements: HashMap<*const syn::Expr, syn::Expr>,
}

impl VisitMut for CaptureReplacer {
    fn visit_expr_mut(&mut self, i: &mut Expr) {
        syn::visit_mut::visit_expr_mut(self, i);
        if let Some(rep) = self.replacements.remove(&(i as _)) {
            *i = rep;
        }
    }
}
