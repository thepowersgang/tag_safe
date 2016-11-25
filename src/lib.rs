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
#![crate_name="tag_safe"]
#![crate_type="dylib"]
#![feature(plugin_registrar, rustc_private)]
#![feature(conservative_impl_trait)]

#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

extern crate syntax;
#[macro_use]
extern crate rustc;
#[macro_use]
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
