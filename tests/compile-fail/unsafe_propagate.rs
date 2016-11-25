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

#[deny(not_tagged_safe)]
#[req_safe(a)]
fn caller() {
	wrapper()
	//~^ ERROR Calling a-unsafe method from
}

