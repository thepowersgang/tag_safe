#![feature(custom_attribute,plugin)]
#![plugin(tag_safe)]
#![allow(dead_code,unused_attributes)]

fn main() {
}


#[tag_unsafe(a)]
fn unsafe_method() {
}

fn wrapper() {
	unsafe_method()
}

#[tag_unsafe(b)]
fn unsafe_b_method() {
	
}
#[tag_unsafe(a,c)]
fn unsafe_ac_method() {
	
}

#[deny(not_tagged_safe)]
#[tag_safe(a,c)]
fn caller() {
	wrapper();
	//~^ ERROR Calling a-unsafe
	unsafe_b_method();
	// don't expect an error
	unsafe_ac_method()
	//~^ ERROR Calling a-unsafe
	//~^^ ERROR Calling c-unsafe
}

