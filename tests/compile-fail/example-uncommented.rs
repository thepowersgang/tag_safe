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
    let _lock = acquire_non_irq_spinlock(&S_NON_IRQ_SPINLOCK);
    //~^ ERROR Calling irq-unsafe method from
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
    irq_handler();
}

// vim: ts=4 sw=4 expandtab
