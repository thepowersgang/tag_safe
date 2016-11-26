# `tag_safe`

[![Build Status](https://travis-ci.org/thepowersgang/tag_safe.svg)](https://travis-ci.org/thepowersgang/tag_safe)

This is a linter designed originally for use with a kernel, where functions need to be marked as "IRQ safe" (meaning they are safe to call
within an IRQ handler, and handle the case where they may interrupt themselves).

# Detailed #
If a function is annotated with `#[req_safe(ident)]` (where `ident` can be anything, and defines the type of safety)
this linter will check that call functions called by that function are either annotated with the same annotation or
`#[is_safe(ident)]`, OR they do no call functions with the reverse `#[is_unsafe(ident)]` annotation.

By default this lint is a warning, if you would like to make it a hard error add `#[deny(not_tagged_safe)]`

Extern crate imports can be annotated with `#[tagged_safe(tag="path/to/list.txt")` to load a list of tagged methods
from an external file. The path is relative to where rustc was invoked (currently), and contains a default tag (true
or false) followed by a newline separated list of methods.

## Example ##
This file annotates all functions in libstd as safe, except for `std::io::_print` (which is the backend for `print!`)
```
true
std::io::_print
```

# Usage #
Below is an example of using this flag to prevent accidentally using an IRQ-unsafe method in an IRQ handler.
(Assume the lock used by `acquire_irq_spinlock` is different to the one acquired by `acquire_non_irq_spinlock`)

```rust
#![feature(custom_attribute,plugin)]
#![plugin(tag_safe)]
/// RAII primitive spinlock
struct Spinlock;
/// Handle to said spinlock
struct HeldSpinlock(&'static Spinlock);
/// RAII IRQ hold
struct IRQLock;
/// Spinlock that also disables IRQs
struct IrqSpinlock(Spinlock);


static S_NON_IRQ_SPINLOCK: Spinlock = Spinlock;
static S_IRQ_SPINLOCK: IrqSpinlock = IrqSpinlock(Spinlock);

#[deny(not_tagged_safe)]	// Make the lint an error
#[req_safe(irq)]	// Require this method be IRQ safe
fn irq_handler()
{
	// The following line would error if it were uncommented, as the
	// acquire_non_irq_spinlock method has been marked as irq-unsafe.
	// If this method was called without protection, the CPU could deadlock.
	//let _lock = acquire_non_irq_spinlock(&S_NON_IRQ_SPINLOCK);
	
	// However, this will not error, this method is marked as IRQ safe
	let _lock = acquire_irq_spinlock(&S_IRQ_SPINLOCK);
}

// This method handles IRQ safety internally, and hence makes
// this lint allowable.
#[is_safe(irq)]
fn acquire_irq_spinlock(l: &'static IrqSpinlock) -> (IRQLock,HeldSpinlock)
{
	// Prevent IRQs from firing
	let irql = hold_irqs();
	// and acquire the spinlock
	(irql, acquire_non_irq_spinlock(&l.0))
}

// Stop IRQs from firing until the returned value is dropped
#[is_safe(irq)]
fn hold_irqs() -> IRQLock
{
	IRQLock
}

// Not safe to call in an IRQ without protection (as that can lead to a
// uniprocessor deadlock)
#[not_safe(irq)]
fn acquire_non_irq_spinlock(l: &'static Spinlock) -> HeldSpinlock
{
	HeldSpinlock(l)
}
```
