use syntax::ast;
use syntax::ast::{ItemKind,TraitItemKind,ImplItemKind};
use syntax::ast::{MetaItemKind,NestedMetaItemKind,LitKind};
use syntax::codemap::Span;
use syntax::ext::base::{MultiItemModifier,MultiItemDecorator};
use syntax::ext::base::{ExtCtxt,Annotatable};
use rustc::lint::LintArray;	// Needed for the lint_array macro


#[derive(Default)]
pub struct HandlerTaggedSafe;
#[derive(Default)]
pub struct HandlerIsSafe;
#[derive(Default)]
pub struct HandlerNotSafe;
//#[derive(Default)]
//pub struct HandlerReqSafe;


impl MultiItemDecorator for HandlerTaggedSafe
{
	fn expand(&self, ecx: &mut ExtCtxt, span: Span, meta_item: &ast::MetaItem, item: &Annotatable, _push: &mut FnMut(Annotatable)) {

		let crate_name = match *item
			{
			//Annotatable::Item(box Item { node: ItemKind::ExternCrate(ref opt_name) }) => {
			Annotatable::Item(ref i) =>
				match i.node {
				ItemKind::ExternCrate(Some(crate_name)) => crate_name,
				_ => return ,
				},
			_ => return ,
			};
		let mut lh = ::database::CACHE.write().expect("Poisoned lock on tag_safe cache");

		let tags = if let MetaItemKind::List(_, ref v) = meta_item.node { v } else { return ; };
		for tag_item_ptr in tags
		{
			let (tag_name, filename) = if let NestedMetaItemKind::MetaItem(ref ptr) = tag_item_ptr.node {
					if let MetaItemKind::NameValue(ref name, ref value_lit) = ptr.node {
						if let LitKind::Str(ref value, _) = value_lit.node {
							(name, value)
						}
						else {
							panic!("");
						}
					}
					else {
						panic!("");
					}
				}
				else {
					panic!("");
				};
			
			let tag = lh.get_tag_or_add(tag_name);
			lh.load_crate(&crate_name.as_str(), tag, filename);
		}
	}
}

fn get_fn_node_id(name: &'static str, item: &Annotatable) -> Option<ast::NodeId> {
	match *item
	{
	Annotatable::Item(ref i) =>
		match i.node
		{
		ItemKind::Fn(..) => Some(i.id),
		_ => {
			warn!("#[{}] on non-fn - {:?}", name, i);
			None
			},
		},
	Annotatable::TraitItem(ref i) =>
		match i.node
		{
		TraitItemKind::Method(..) => Some(i.id),
		_ => {
			warn!("#[{}] on non-fn - {:?}", name, i);
			None
			},
		},
	Annotatable::ImplItem(ref i) =>
		match i.node
		{
		ImplItemKind::Method(..) => Some(i.id),
		_ => {
			warn!("#[{}] on non-fn - {:?}", name, i);
			None
			},
		},
	}
}

impl MultiItemDecorator for HandlerIsSafe
{
	fn expand(&self, ecx: &mut ExtCtxt, span: Span, meta_item: &ast::MetaItem, item: &Annotatable, _push: &mut FnMut(Annotatable)) {
		let node_id = match get_fn_node_id("is_safe", item)
			{
			Some(v) => v,
			None => return,
			};
		let mut lh = ::database::CACHE.write().expect("Poisoned lock on tag_safe cache");
		for tag_name in get_tags(ecx, meta_item, "is_safe")
		{
			debug!("#[is_safe] {} - {}", tag_name, node_id);
			/*let tag = */lh.get_tag_or_add(tag_name);
			//lh.mark(node_id, tag, true);
		}
	}
}

impl MultiItemDecorator for HandlerNotSafe
{
	fn expand(&self, ecx: &mut ExtCtxt, span: Span, meta_item: &ast::MetaItem, item: &Annotatable, _push: &mut FnMut(Annotatable)) {
		let node_id = match get_fn_node_id("not_safe", item)
			{
			Some(v) => v,
			None => return,
			};
		let mut lh = ::database::CACHE.write().expect("Poisoned lock on tag_safe cache");
		for tag_name in get_tags(ecx, meta_item, "not_safe")
		{
			debug!("#[not_safe] {} - {}", tag_name, node_id);
			/*let tag = */lh.get_tag_or_add(tag_name);
			//lh.mark(node_id, tag, false);
		}
	}
}

//impl MultiItemDecorator for HandlerReqSafe
//{
//	fn expand(&self, ecx: &mut ExtCtxt, span: Span, meta_item: &ast::MetaItem, item: &Annotatable, _push: &mut FnMut(Annotatable)) {
//		let node_id = match get_fn_node_id("req_safe", item)
//			{
//			Some(v) => v,
//			None => return,
//			};
//		let mut lh = ::database::CACHE.write().expect("Poisoned lock on tag_safe cache");
//		for tag_name in get_tags(ecx, meta_item, "req_safe")
//		{
//			debug!("#[req_safe] {} - {}", tag_name, node_id);
//			let tag = lh.get_tag_or_add(tag_name);
//			lh.mark(node_id, tag, true);
//		}
//	}
//}

fn get_tags<'a>(_cx: &'a ExtCtxt, meta_item: &'a ast::MetaItem, attr_name: &'a str) -> impl Iterator<Item=&'a str>+'a {
	if let MetaItemKind::List(_, ref v) = meta_item.node {
		v.iter().filter_map(|tag_meta|
			if let NestedMetaItemKind::MetaItem(ref ptr) = tag_meta.node {
				if let MetaItemKind::Word(ref name) = ptr.node {
					Some(&**name)
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
	else {
		panic!("");
	}
}

