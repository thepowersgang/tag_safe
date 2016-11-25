#![feature(custom_attribute,plugin)]
#![plugin(tag_safe)]
#![allow(dead_code)]

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

#[deny(not_tagged_safe)]    // Make the lint an error
#[req_safe(irq)]    // Require this method be IRQ safe
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

// Not safe to call in an IRQ without protection (as that can lead to a uniprocessor deadlock)
#[not_safe(irq)]
fn acquire_non_irq_spinlock(l: &'static Spinlock) -> HeldSpinlock
{
    HeldSpinlock(l)
}

fn main() {
}
