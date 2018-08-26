// tag_safe
//
// A linting plugin to flag calls to methods not marked "tag_safe"
// from methods marked "tag_safe".
//
// Author: John Hodge (thePowersGang/Mutabah)
//
// TODO: Support '#[tag_unsafe(type)]' which is used when a method has no marker
// - Allows default safe fallback, with upwards propagation.
//
//! 
//! Provides a lint that warns/errors when a function calls a method that is marked with a
//! particular kind of unsafety.
//! 
//! Four attributes are used to allow functions to be marked.
//! - `#[not_safe(tags)]` - Marks a function as not being safe for the given tags
//! - `#[is_safe(tags)]` - Marks the function as being safe for the given tags (despite what it does internally)
//! - `#[tagged_safe(tag="file")]` Loads a list of tagged functions for an extern crate from a file.
//! - `#[req_safe(tags)]` - Enables linting this function for use the given tags
//!
#![crate_name="tag_safe"]
#![crate_type="dylib"]
#![feature(plugin_registrar, rustc_private)]
#![feature(macro_at_most_once_rep)] // also needed for a macro (lint_array)

#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

extern crate syntax;
#[macro_use]
extern crate rustc;
extern crate rustc_plugin;

mod prescan;
mod check;
mod database;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut ::rustc_plugin::Registry) {
    use syntax::feature_gate::AttributeType;
    use syntax::ext::base::SyntaxExtension;
    use syntax::symbol::Symbol;
    
    reg.register_syntax_extension(Symbol::intern("tagged_safe"), SyntaxExtension::MultiDecorator(Box::new(prescan::HandlerTaggedSafe)) );
    //reg.register_syntax_extension(intern("is_safe" ), SyntaxExtension::MultiDecorator(Box::new(prescan::HandlerIsSafe) ) );
    //reg.register_syntax_extension(intern("not_safe"), SyntaxExtension::MultiDecorator(Box::new(prescan::HandlerNotSafe)) );
    //reg.register_syntax_extension(intern("req_safe"), SyntaxExtension::MultiModifier(Box::new(prescan::HandlerReqSafe)) );

    reg.register_late_lint_pass( Box::new(check::Pass::default()) );

    reg.register_attribute(String::from("is_safe" ), AttributeType::Whitelisted);
    reg.register_attribute(String::from("not_safe"), AttributeType::Whitelisted);
    reg.register_attribute(String::from("req_safe"), AttributeType::Whitelisted);
}

// vim: ts=4 expandtab sw=4
