# tag_safe

[![Build Status](https://travis-ci.org/thepowersgang/tag_safe.svg)](https://travis-ci.org/thepowersgang/tag_safe)

This is a linter designed originally for use with a kernel, where functions need to be marked as "IRQ safe" (meaning they are safe to call
within an IRQ handler, and handle the case where they may interrupt themselves).

# Detailed #
If a function is annotated with `#[tag_safe(ident)]` (where `ident` can be anything, and defines the type of safety) this linter will check that call functions called by that function either have that same annotation, or don't call any function with the reverse `#[tag_unsafe(ident)]` annotation.

By default this lint is a warning, in functions that internally ensure safety it can be turned off with `#[allow(not_tagged_safe)]`, and for functions that require safety it can be made an error with `#[deny(not_tagged_safe)]`

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
#[tag_safe(irq)]	// Require this method be IRQ safe
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
#[tag_safe(irq)]
#[allow(not_tagged_safe)]
fn acquire_irq_spinlock(l: &'static IrqSpinlock) -> (IRQLock,HeldSpinlock)
{
	// Prevent IRQs from firing
	let irql = hold_irqs();
	// and acquire the spinlock
	(irql, acquire_non_irq_spinlock(&l.0))
}

// Stop IRQs from firing until the returned value is dropped
#[tag_safe(irq)]
fn hold_irqs() -> IRQLock
{
	IRQLock
}

// Not safe to call in an IRQ without protection (as that can lead to a
// uniprocessor deadlock)
#[tag_unsafe(irq)]
fn acquire_non_irq_spinlock(l: &'static Spinlock) -> HeldSpinlock
{
	HeldSpinlock(l)
}
```
