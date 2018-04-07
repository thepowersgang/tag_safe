use syntax::ast;
use syntax::ast::{ItemKind,TraitItemKind,ImplItemKind};
use syntax::ast::{MetaItemKind,NestedMetaItemKind,LitKind};
use syntax::codemap::Span;
use syntax::ext::base::{MultiItemModifier,MultiItemDecorator};
use syntax::ext::base::{ExtCtxt,Annotatable};


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
				ItemKind::ExternCrate(None) => i.ident.name,
				ItemKind::ExternCrate(Some(crate_name)) => crate_name,
				_ => return ,
				},
			_ => return ,
			};
		let mut lh = ::database::CACHE.write().expect("Poisoned lock on tag_safe cache");

		for (tag_name, filename) in get_inner_items(meta_item, "tagged_safe")
			.filter_map(|ptr| 
				if let MetaItemKind::NameValue( ast::Lit { node: LitKind::Str(ref value, _), .. } ) = ptr.node {
					Some( (ptr.ident.name, value) )
				}
				else {
					warn!("");
					None
				}
				)
		{
			let tag = lh.get_tag_or_add(&tag_name.as_str());
			match lh.load_crate(&crate_name.as_str(), tag, &filename.as_str())
			{
			Ok(_) => {},
			Err(e) => {
				ecx.span_err(span, &format!("Couldn't open tagging list file from '{}' - {}", filename.as_str(), e));
				},
			}
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
	Annotatable::ForeignItem(_) => {
		None
		},
	Annotatable::Stmt(_) => {
		None
		},
	Annotatable::Expr(_) => {
		None
		},
	}
}

impl MultiItemDecorator for HandlerIsSafe
{
	fn expand(&self, ecx: &mut ExtCtxt, _span: Span, meta_item: &ast::MetaItem, item: &Annotatable, _push: &mut FnMut(Annotatable)) {
		let node_id = match get_fn_node_id("is_safe", item)
			{
			Some(v) => v,
			None => return,
			};
		let mut lh = ::database::CACHE.write().expect("Poisoned lock on tag_safe cache");
		for tag_name in get_tags(ecx, meta_item, "is_safe")
		{
			debug!("#[is_safe] {} - {}", tag_name, node_id);
			/*let tag = */lh.get_tag_or_add(&tag_name.as_str());
			//lh.mark(node_id, tag, true);
		}
	}
}

impl MultiItemDecorator for HandlerNotSafe
{
	fn expand(&self, ecx: &mut ExtCtxt, _span: Span, meta_item: &ast::MetaItem, item: &Annotatable, _push: &mut FnMut(Annotatable)) {
		let node_id = match get_fn_node_id("not_safe", item)
			{
			Some(v) => v,
			None => return,
			};
		let mut lh = ::database::CACHE.write().expect("Poisoned lock on tag_safe cache");
		for tag_name in get_tags(ecx, meta_item, "not_safe")
		{
			debug!("#[not_safe] {} - {}", tag_name, node_id);
			/*let tag = */lh.get_tag_or_add(&tag_name.as_str());
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

fn get_inner_items<'a>(meta_item: &'a ast::MetaItem, attr_name: &'a str) -> impl Iterator<Item=&'a ast::MetaItem>+'a {
	let it = if let MetaItemKind::List(ref v) = meta_item.node {
			v.iter()
		}
		else {
			warn!("Attribute '{}' must take a list", attr_name);
			[].iter()
		};
	it.filter_map(|tag_meta|
		if let NestedMetaItemKind::MetaItem(ref ptr) = tag_meta.node {
			Some(ptr)
		}
		else {
			warn!("");
			None
		}
		)
}

fn get_tags<'a>(_cx: &'a ExtCtxt, meta_item: &'a ast::MetaItem, attr_name: &'a str) -> impl Iterator<Item=::syntax::symbol::Symbol>+'a {
	get_inner_items(meta_item, attr_name)
		.filter_map(|ptr|
			if let MetaItemKind::Word = ptr.node {
				Some(ptr.ident.name)
			}
			else {
				warn!("");
				None
			}
			)
}

