
use std::sync::RwLock;
use std::collections::{HashMap,hash_map};
use syntax::ast;
use rustc::hir::def_id;

#[derive(Default)]
pub struct StaticCache
{
	known_tags: Vec<String>,
	this_crate: AnnotationCache,
	//ext_crates: HashMap<CrateNum, AnnotationCache>,
	ext_crates: HashMap<String, AnnotationCache>,
}

#[derive(Default)]
struct AnnotationCache
{
	// tag -> cache
	map: HashMap<usize, TagAnnotationCache>,
}

#[derive(Default)]
struct TagAnnotationCache
{
	// functon -> state
	map: HashMap<ast::NodeId, bool>,
}

#[derive(Copy,Clone)]
pub struct Tag(usize);

lazy_static! {
	// RwLock becuase after building, this will be uncontended.
	pub static ref CACHE: RwLock<StaticCache> = Default::default();
}

impl StaticCache
{
	pub fn get_tag_or_add(&mut self, tag_name: &str) -> Tag {
		if let Some(i) = self.known_tags.iter().position(|x| x == tag_name) {
			Tag(i)
		}
		else {
			let i = self.known_tags.len();
			self.known_tags.push( tag_name.to_string() );
			Tag(i)
		}
	}
	pub fn get_tag_opt(&self, tag_name: &str) -> Option<Tag> {
		self.known_tags.iter()
			.position(|x| x == tag_name)
			.map(|i| Tag(i))
	}

	pub fn mark(&mut self, id: ast::NodeId, tag: Tag, is_safe: bool) {
		let tag_cache = self.this_crate.map.entry(tag.0).or_insert_with(|| Default::default());
		match tag_cache.map.entry(id)
		{
		hash_map::Entry::Occupied(_) => {},
		hash_map::Entry::Vacant(e) => { e.insert(is_safe); },
		}
	}
	pub fn load_crate(&mut self, crate_name: &str, tag: Tag, filename: &str) {
		// TODO:
	}

	pub fn get_local(&self, id: ast::NodeId, tag: Tag) -> Option<bool> {
		self.this_crate.map.get(&tag.0)
			.and_then(|tc| tc.map.get(&id))
			.map(|&v| v)
	}
	pub fn get_extern(&self, krate: def_id::CrateNum, index: def_id::DefIndex, tag: Tag) -> Option<bool> {
		panic!("TODO: get_extern");
	}
}

