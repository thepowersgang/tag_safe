
use syntax::ast;
use syntax::ast::{MetaItemKind,NestedMetaItemKind};
use rustc::hir::def_id::DefId;
use rustc::hir::def;
use syntax::codemap::Span;
use rustc::lint::{self, LintContext, LintPass, LateLintPass, LintArray};
use rustc::ty::{self, TyCtxt};
use rustc::hir;

declare_lint!(NOT_TAGGED_SAFE, Warn, "Warn about use of non-tagged methods within tagged function");

#[derive(Default)]
pub struct Pass
{
    visit_stack: Vec<ast::NodeId>,
}

impl LintPass for Pass {
    fn get_lints(&self) -> LintArray {
        lint_array!(NOT_TAGGED_SAFE)
    }
}

impl<'a,'b> LateLintPass<'a,'b> for Pass {
    fn check_fn(&mut self, cx: &lint::LateContext, _kind: hir::intravisit::FnKind, _decl: &hir::FnDecl, body: &hir::Body, _: Span, id: ast::NodeId) {
        let attrs = cx.tcx.map.attrs(id);

        // If this function is tagged with a particular safety, store
        {
            let mut lh = ::database::CACHE.write().unwrap();
            for tag_name in get_tags(attrs, "is_safe")
            {
                let tag = lh.get_tag_or_add(&tag_name.as_str());
                lh.mark(id, tag,  true);
            }
            for tag_name in get_tags(attrs, "not_safe")
            {
                let tag = lh.get_tag_or_add(&tag_name.as_str());
                lh.mark(id, tag,  false);
            }
        }
        
        // For each required safety, check
        for tag_name in get_tags(attrs, "req_safe")
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
    fn fill_cache_for(&mut self, tcx: &TyCtxt, node_id: ast::NodeId)
    {
        debug!("Filling cache for node {:?}", node_id);
        let attrs = tcx.map.attrs(node_id);
        let mut lh = ::database::CACHE.write().unwrap();
        for tag_name in Iterator::chain( get_tags(attrs, "is_safe"), get_tags(attrs, "req_safe") )
        {
            debug!("#[is_safe/req_safe] {} - {}", tag_name, node_id);
            let tag = lh.get_tag_or_add(&tag_name.as_str());
            lh.mark(node_id, tag,  true);
        }
        for tag_name in get_tags(attrs, "not_safe")
        {
            debug!("#[not_safe] {} - {}", tag_name, node_id);
            let tag = lh.get_tag_or_add(&tag_name.as_str());
            lh.mark(node_id, tag,  false);
        }
    }

    /// Recursively check that the provided function is either safe or unsafe.
    // Used to avoid excessive annotating
    fn recurse_fcn_body(&mut self, cx: &lint::LateContext, node_id: ast::NodeId, tag: ::database::Tag) -> bool
    {
        // and apply a visitor to all 
        match cx.tcx.map.get(node_id)
        {
        hir::map::NodeItem(i) =>
            match i.node {
            hir::ItemFn(_, _, _, _, _, ref body) => {
                // Enumerate this function's code, recursively checking for a call to an unsafe method
                let mut is_safe = true;
                {
                    let mut v = Visitor {
                        pass: self, cx: cx, tag: tag,
                        cb: |_| { is_safe = false; }
                        };
                    hir::intravisit::walk_body(&mut v, cx.tcx.map.body(*body));
                }
                is_safe
                },
            ref v @ _ => {
                error!("Node ID {} points to a non-function item {:?}", node_id, v);
                true
                },
            },
        hir::map::NodeImplItem(i) =>
            match i.node {
            hir::ImplItemKind::Method(_, ref body) => {
                
                let mut is_safe = true;
                {
                    let mut v = Visitor {
                        pass: self, cx: cx, tag: tag,
                        cb: |_| { is_safe = false; }
                        };
                    hir::intravisit::walk_body(&mut v, cx.tcx.map.body(*body));
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
        if ! id.is_local()
        {
            // TODO: Get the entry from the crate cache
            if let Some(v) = ::database::CACHE.read().unwrap().get_extern(&cx.tcx,id.krate, id.index, tag) {
                debug!("{:?} - {} (extern cached)", id, v);
                v
            }
            else {
                debug!("{:?} - {} (extern assumed)", id, true);
                true
            }
        }
        else
        {
            let node_id = cx.tcx.map.as_local_node_id(id).expect("Unable to locate node");
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
        match ex.node
        {
        // Call expressions - check that it's a path call
        hir::ExprCall(ref fcn, ..) =>
			match fcn.node
			{
			hir::ExprPath(ref qp, ..) =>
				match self.cx.tables.qpath_def(qp, fcn.id)
				{
				def::Def::Fn(did) | def::Def::Method(did) =>
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
        hir::ExprMethodCall(ref _id, ref _tys, ref _exprs) =>
			match self.cx.tables.method_map.get( &ty::MethodCall::expr(ex.id) )
			{
			Some(callee) => {
                let id = callee.def_id;
                
				// Check for a safety tag
				if !self.pass.method_is_safe(self.cx, id, self.tag) {
					(self.cb)(&ex.span);
				}
				},
			None => info!("ExprMethodCall with unknown callee"),
			},
        
        // Ignore any other type of node
        _ => {},
        }
        hir::intravisit::walk_expr(self, ex);
    }
}

fn get_tags<'a>(meta_items: &'a [ast::Attribute], attr_name: &'a str) -> impl Iterator<Item=::syntax::symbol::Symbol>+'a {
    meta_items.iter()
        .filter(move |attr| attr.value.name == attr_name)
        .flat_map(|attr|
            if let MetaItemKind::List(ref v) = attr.value.node {
                v.iter()
            }
            else {
                panic!("");
            }
            )
        .filter_map(|tag_meta|
            if let NestedMetaItemKind::MetaItem(ref ptr) = tag_meta.node {
                if let MetaItemKind::Word = ptr.node {
                    Some(ptr.name)
                }
                else {
                    warn!("");
                    None
                }
            }
            else {
                warn!("");
                None
            }
            )
}
