This is a linter designed originally for use with a kernel, where functions need to be marked as "IRQ safe" (meaning they are safe to call
within an IRQ handler, and handle the case where they may interrupt themselves).

# Detailed #
Within a function marked with a `#[tag_safe(type)]` annotation, the linter will check all called functions that are also marked as `#[tag_safe(type)]`.

# Usage #
Below is an example of using this flag to prevent accidentally using an IRQ-unsafe method in an IRQ handler.
(Assume the lock used by `acquire_irq_spinlock` is different to the one acquired by `acquire_non_irq_spinlock`)

```rust
/// RAII IRQ disabler
struct IRQLock;

/// RAII primitive spinlock
struct HeldSpinlock;

#[deny(not_tagged_safe)]	// Make the lint an error
#[tag_safe(irq)]	// Require this method be IRQ safe
fn irq_handler()
{
	// The following line would error if it were uncommented, as the
	// acquire_non_irq_spinlock method has been marked as irq-unsafe.
	// If this method was called without protection, the CPU could deadlock.
	//let _lock = acquire_non_irq_spinlock();
	
	// However, this will not error, this method is marked as IRQ safe
	let _lock = acquire_irq_spinlock();
}

// This method handles IRQ safety internally, and hence makes
// this lint allowable.
#[tag_safe(irq)]
#[allow(not_tagged_safe)]
fn acquire_irq_spinlock() -> (IRQLock,HeldSpinlock)
{
	// Prevent IRQs from firing
	let irql = hold_irqs();
	// and acquire the spinlock
	(irql, acquire_non_irq_spinlock())
}

// Stop IRQs from firing until the returned value is dropped
#[tag_safe(irq)]
fn hold_irqs() -> IRQLock
{
	IRQLock
}

#[tag_unsafe(irq)]
fn acquire_non_irq_spinlock() -> HeldSpinlock
{
	HeldSpinlock
}
```
