#![feature(custom_attribute,plugin)]
#![plugin(tag_safe)]
#![allow(dead_code)]

fn main() {
}


#[not_safe(a)]
fn unsafe_method() {
}

fn wrapper() {
	unsafe_method()
}

#[not_safe(b)]
fn unsafe_b_method() {
	
}
#[not_safe(a,c)]
fn unsafe_ac_method() {
	
}

#[deny(not_tagged_safe)]
#[req_safe(a,c)]
fn caller() {
	wrapper();
	//~^ ERROR Calling a-unsafe
	unsafe_b_method();
	// don't expect an error
	unsafe_ac_method()
	//~^ ERROR Calling a-unsafe
	//~^^ ERROR Calling c-unsafe
}

