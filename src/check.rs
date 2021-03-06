
use syntax::ast;
use syntax::ast::{MetaItemKind,NestedMetaItem};
use rustc::hir::def_id::DefId;
use rustc::hir::def;
use syntax::source_map::{Span, Symbol};
use rustc::lint::{self, LintContext, LintPass, LateLintPass, LintArray};
use rustc::ty::{TyCtxt};
use rustc::hir::{self, ExprKind, ItemKind};

declare_lint!(NOT_TAGGED_SAFE, Warn, "Warn about use of non-tagged methods within tagged function");

pub struct Pass
{
    visit_stack: Vec<hir::HirId>,
	pub sym_issafe: Symbol,
	pub sym_notsafe: Symbol,
	pub sym_reqsafe: Symbol,
}
impl Pass
{
	pub fn new() -> Self
	{
		Pass {
			visit_stack: Vec::new(),
			sym_issafe: Symbol::intern("is_safe"),
			sym_notsafe: Symbol::intern("not_safe"),
			sym_reqsafe: Symbol::intern("req_safe"),
			}
	}
}

impl LintPass for Pass {
	fn name(&self) -> &'static str {
		"tag_safe"
	}
    fn get_lints(&self) -> LintArray {
        lint_array!(NOT_TAGGED_SAFE)
    }
}

impl<'a,'b> LateLintPass<'a,'b> for Pass {
    fn check_fn(&mut self, cx: &lint::LateContext, _kind: hir::intravisit::FnKind, _decl: &hir::FnDecl, body: &hir::Body, _: Span, id: hir::HirId) {
        let attrs = cx.tcx.hir().attrs(id);

        // If this function is tagged with a particular safety, store
        {
            let mut lh = ::database::CACHE.write().unwrap();
            for tag_name in get_tags(attrs, self.sym_issafe.clone())
            {
                let tag = lh.get_tag_or_add(&tag_name.as_str());
                lh.mark(id, tag,  true);
            }
            for tag_name in get_tags(attrs, self.sym_notsafe.clone())
            {
                let tag = lh.get_tag_or_add(&tag_name.as_str());
                lh.mark(id, tag,  false);
            }
        }
        
        // For each required safety, check
        for tag_name in get_tags(attrs, self.sym_reqsafe.clone())
        {
            let ty_tag = {
                let mut lh = ::database::CACHE.write().unwrap();
                let tag = lh.get_tag_or_add(&tag_name.as_str());
                //let tag = if let Some(v) = lh.get_tag_opt(&tag_name.as_str()) { v } else { error!("Tag {} unknown", ty_name);  continue; };
                lh.mark(id, tag,  true);
                tag
                };

            // Search body for calls to non safe methods
            let mut v = Visitor {
                    pass: self, cx: cx, tag: ty_tag,
                    cb: |span| {
                            cx.span_lint(NOT_TAGGED_SAFE, *span,
                                &format!("Calling {0}-unsafe method from a #[req_safe({0})] method", tag_name)[..]
                                );
                        },
                    };
            debug!("Method {:?} is marked safe '{}'", id, tag_name);
            hir::intravisit::walk_body(&mut v, body);
        }

        // TODO: For all known safeties (that aren't already set) populate.
        // - Requires knowing all safeties (which we don't, ... yet)
    }
}

impl Pass
{
    fn fill_cache_for(&mut self, tcx: &TyCtxt, node_id: hir::HirId)
    {
        debug!("Filling cache for node {:?}", node_id);
        let attrs = tcx.hir().attrs(node_id);
        let mut lh = ::database::CACHE.write().unwrap();
        for tag_name in Iterator::chain( get_tags(attrs, self.sym_issafe.clone()), get_tags(attrs, self.sym_reqsafe.clone()) )
        {
            debug!("#[is_safe/req_safe] {} - {}", tag_name, node_id);
            let tag = lh.get_tag_or_add(&tag_name.as_str());
            lh.mark(node_id, tag,  true);
        }
        for tag_name in get_tags(attrs, self.sym_notsafe.clone())
        {
            debug!("#[not_safe] {} - {}", tag_name, node_id);
            let tag = lh.get_tag_or_add(&tag_name.as_str());
            lh.mark(node_id, tag,  false);
        }
    }

    /// Recursively check that the provided function is either safe or unsafe.
    // Used to avoid excessive annotating
    fn recurse_fcn_body(&mut self, cx: &lint::LateContext, node_id: hir::HirId, tag: ::database::Tag) -> bool
    {
        // and apply a visitor to all 
        match cx.tcx.hir().get(node_id)
        {
        hir::Node::Item(i) =>
            match i.kind
			{
            ItemKind::Fn(_, _, _, ref body) => {
                // Enumerate this function's code, recursively checking for a call to an unsafe method
                let mut is_safe = true;
                {
                    let mut v = Visitor {
                        pass: self, cx: cx, tag: tag,
                        cb: |_| { is_safe = false; }
                        };
                    hir::intravisit::walk_body(&mut v, cx.tcx.hir().body(*body));
                }
                is_safe
                },
            ref v @ _ => {
                error!("Node ID {} points to a non-function item {:?}", node_id, v);
                true
                },
            },
        hir::Node::ImplItem(i) =>
            match i.kind
			{
            hir::ImplItemKind::Method(_, ref body) => {
                
                let mut is_safe = true;
                {
                    let mut v = Visitor {
                        pass: self, cx: cx, tag: tag,
                        cb: |_| { is_safe = false; }
                        };
                    hir::intravisit::walk_body(&mut v, cx.tcx.hir().body(*body));
                }
                is_safe
                },
            _ => true,
            },
        //hir::map::NodeForeignItem(i) =>
        //    if Self::check_for_marker(tcx, i.id, "tag_safe", name) {
        //        true
        //    }
        //    else if Self::check_for_marker(tcx, i.id, "tag_unsafe", name) {
        //        false
        //    }
        //    else {
        //        unknown_assume
        //    },
        ref v @ _ => {
            error!("Node ID {} points to non-item {:?}", node_id, v);
            true
            }
        }
    }
    
    /// Locate a #[tag_safe(<name>)] attribute on the passed item
    pub fn method_is_safe(&mut self, cx: &lint::LateContext, id: DefId, tag: ::database::Tag) -> bool
    {
        match cx.tcx.hir().as_local_hir_id(id)
        {
        None => {
            // TODO: Get the entry from the crate cache
            if let Some(v) = ::database::CACHE.read().unwrap().get_extern(&cx.tcx,id.krate, id.index, tag) {
                debug!("{:?} - {} (extern cached)", id, v);
                v
            }
            else {
                debug!("{:?} - {} (extern assumed)", id, true);
                true
            }
            },
        Some(node_id) => {
            let mut local_opt = ::database::CACHE.read().unwrap().get_local(node_id, tag);
            // NOTE: This only fires once (ideally)
            if local_opt.is_none() {
                self.fill_cache_for(&cx.tcx, node_id);
                local_opt = ::database::CACHE.read().unwrap().get_local(node_id, tag);
            }
            if let Some(v) = local_opt {
                debug!("{} - {} (cached)", node_id, v);
                v
            }
            else {
                // If this node is currently being checked, assume it's valid.
                // TODO: This can lead to a false positive being stored.
                if self.visit_stack.iter().position(|x| *x == node_id).is_some() {
                    warn!("Recursion, assuming true");
                    true
                }
                else {
                    self.visit_stack.push(node_id);
                    let rv = self.recurse_fcn_body(cx, node_id, tag);
                    self.visit_stack.pop();
                    
                    debug!("{} - {} (recursed)", node_id, rv);
                    ::database::CACHE.write().unwrap().mark(node_id, tag,  rv);
                    rv
                }
            }
            }
        }
    }
}

struct Visitor<'a, 'tcx: 'a, F: FnMut(&Span) + 'a>
{
    pass: &'a mut Pass,
	cx: &'a lint::LateContext<'a,'tcx>,
    tag: ::database::Tag,
    cb: F,
}

impl<'a, 'tcx: 'a, F: FnMut(&Span)> hir::intravisit::Visitor<'a> for Visitor<'a, 'tcx, F>
{
	fn nested_visit_map<'this>(&'this mut self) -> hir::intravisit::NestedVisitorMap<'this, 'a> {
		hir::intravisit::NestedVisitorMap::None
	}

    // Locate function/method calls in a code block
    fn visit_expr(&mut self, ex: &'a hir::Expr) {
        debug!("visit node - {:?}", ex);
        match ex.kind
        {
        // Call expressions - check that it's a path call
        ExprKind::Call(ref fcn, ..) =>
			match fcn.kind
			{
			ExprKind::Path(ref qp, ..) =>
				match self.cx.tables.qpath_res(qp, fcn.hir_id)
				{
				def::Res::Def(def::DefKind::Fn, did) | def::Res::Def(def::DefKind::Method, did) =>
					// Check for a safety tag
					if !self.pass.method_is_safe(self.cx, did, self.tag)
					{
						(self.cb)(&ex.span);
					}
					else {
						debug!("Safe call {:?}", ex);
					},
				_ => {
					info!("Call ExprPath with an unknown Def type");
					},
				},
			_ => {
				info!("Call without ExprPath");
				},
			},
        
        // Method call expressions - get the relevant method
        ExprKind::MethodCall(ref _id, ref _tys, ref _exprs) =>
			match self.cx.tables.type_dependent_defs().get(ex.hir_id)
			{
			Some(Ok(callee)) => {
                let id = callee.1;
                
				// Check for a safety tag
				if !self.pass.method_is_safe(self.cx, id, self.tag) {
					(self.cb)(&ex.span);
				}
				},
			_ => info!("ExprMethodCall with unknown callee"),
			},
        
        // Ignore any other type of node
        _ => {},
        }
        hir::intravisit::walk_expr(self, ex);
    }
}

fn get_tags<'a>(meta_items: &'a [ast::Attribute], attr_name: Symbol) -> impl Iterator<Item=::syntax::symbol::Symbol>+'a {
    meta_items.iter()
        .filter(move |attr| attr.path == attr_name)
        .flat_map(|attr|
			if let Some(v) = attr.meta() {
				if let MetaItemKind::List(v) = v.kind {
					v.into_iter()
				}
				else {
					panic!("");
				}
			}
			else {
				panic!("");
			}
            )
        .filter_map(|tag_meta|
            if let NestedMetaItem::MetaItem(ref ptr) = tag_meta {
				match (&ptr.kind, ptr.ident())
				{
				(&MetaItemKind::Word, Some(i)) => Some(i.name),
				_ => {
                    warn!("");
                    None
					}
				}
            }
            else {
                warn!("");
                None
            }
            )
}
