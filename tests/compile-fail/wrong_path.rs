#![feature(plugin)]
#![plugin(tag_safe)]

#[tagged_safe(foo="some_missing_file")]
//~^ ERROR Couldn't open tagging list file
extern crate core;

#[tagged_safe(foo="some_missing_file")]
//~^ ERROR Couldn't open tagging list file
extern crate core as core_;

fn main() {
}

