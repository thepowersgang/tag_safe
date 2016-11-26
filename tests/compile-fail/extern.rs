#![feature(plugin)]
#![plugin(tag_safe)]

// Path is relative to rustc invocation directory - package root in this case
#[tagged_safe(print="tests/compile-fail/libstd_foo.txt")]
extern crate std as _std;

#[req_safe(print)]
#[deny(not_tagged_safe)]
fn main() {
	bar();
    //~^ ERROR Calling print-unsafe method from
}

fn bar() {
	println!("Hello World");
}

