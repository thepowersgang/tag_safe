#![feature(plugin)]
#![plugin(tag_safe)]
#![deny(not_tagged_safe)]

#[req_safe(foo)]
fn main() {
	let v = Foo::new();
	//~^ ERROR Calling foo-unsafe
	v.method();
	//~^ ERROR Calling foo-unsafe
}

struct Foo;
impl Foo
{
	#[not_safe(foo)]
	fn new() -> Foo { Foo }
	
	#[not_safe(foo)]
	fn method(&self) {}
}

