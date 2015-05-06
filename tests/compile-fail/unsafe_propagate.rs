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

#[deny(not_tagged_safe)]
#[tag_safe(a)]
fn caller() {
	wrapper()
	//~^ ERROR Calling a-unsafe method from
}

