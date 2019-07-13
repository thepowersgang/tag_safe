
use std::sync::RwLock;
use std::collections::{HashMap,HashSet,hash_map};
use rustc::hir;
use rustc::hir::def_id;
use rustc::ty::TyCtxt;

#[derive(Default)]
pub struct StaticCache
{
	known_tags: Vec<String>,
	this_crate: AnnotationCache,
	//ext_crates: HashMap<CrateNum, AnnotationCache>,
	ext_crates: HashMap<String, ExtCache>,
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
	map: HashMap<hir::HirId, bool>,
}

#[derive(Default)]
struct ExtCache
{
	// tag -> cache
	tag_map: HashMap<usize, ExtTagCache>,
}
#[derive(Default)]
struct ExtTagCache
{
	default: bool,
	// name -> inner ID mapping
	name_set: HashSet<String>,
	// def_id -> inner ID mapping
	id_map: RwLock< HashMap<def_id::DefIndex,bool> >,

	//map: Vec<bool>,
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
	//pub fn get_tag_opt(&self, tag_name: &str) -> Option<Tag> {
	//	self.known_tags.iter()
	//		.position(|x| x == tag_name)
	//		.map(|i| Tag(i))
	//}

	pub fn mark(&mut self, id: hir::HirId, tag: Tag, is_safe: bool) {
		let tag_cache = self.this_crate.map.entry(tag.0).or_insert_with(|| Default::default());
		match tag_cache.map.entry(id)
		{
		hash_map::Entry::Occupied(_) => {},
		hash_map::Entry::Vacant(e) => { e.insert(is_safe); },
		}
	}
	pub fn load_crate(&mut self, crate_name: &str, tag: Tag, filename: &str) -> Result<(),::std::io::Error> {
		use ::std::io::BufRead;
		let mut fp = match ::std::fs::File::open(filename)
			{
			Ok(v) => ::std::io::BufReader::new(v),
			Err(e) => {
				error!("Cannot open file '{}'", filename);
				return Err(e);
				},
			};
		// Line 1: Default
		let default = {
			let mut line = String::new();
			fp.read_line(&mut line)?;
			match line.trim()
			{
			"true" => true,
			"false" => false,
			_ => return Err(::std::io::Error::new(::std::io::ErrorKind::InvalidData, "First line wasn't `true` or `false`")),
			}
			};
		let cache = match self.ext_crates.entry(String::from(crate_name)).or_insert_with(|| Default::default()).tag_map.entry(tag.0)
			{
			hash_map::Entry::Occupied(_) => return Ok( () ),
			hash_map::Entry::Vacant(e) => e.insert(ExtTagCache { default: default, ..Default::default() }),
			};
		// Rest: Entries
		for line in fp.lines()
		{
			cache.name_set.insert( line? );
		}
		Ok( () )
	}

	pub fn get_local(&self, id: hir::HirId, tag: Tag) -> Option<bool> {
		self.this_crate.map.get(&tag.0)
			.and_then(|tc| tc.map.get(&id))
			.map(|&v| v)
	}
	pub fn get_extern(&self, tcx: &TyCtxt, krate: def_id::CrateNum, index: def_id::DefIndex, tag: Tag) -> Option<bool> {
		let cache = match self.ext_crates.get(&*tcx.crate_name(krate).as_str()).and_then(|c| c.tag_map.get(&tag.0))
			{
			None => return None,
			Some(e) => e,
			};
		if let Some(v) = cache.id_map.read().unwrap().get(&index)
		{
			return Some(*v);
		}
		
		match cache.id_map.write().unwrap().entry(index)
		{
		hash_map::Entry::Occupied(e) => Some(*e.get()),
		hash_map::Entry::Vacant(e) => {
			let name = tcx.def_path_str(def_id::DefId{krate:krate,index:index});
			debug!("Look up {}", name);
			if cache.name_set.contains( &name ) {
				Some(*e.insert( !cache.default ))
			}
			else {
				Some(*e.insert( cache.default ))
			}
			},
		}
	}
}

