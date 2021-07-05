/*
 * This helper is to make it simpler to Box a signal-factory function
 * The use case is similar to where you might have a ReadOnlyMutable or Broadcaster
 * e.g. to re-render the same signal multiple times
 * If the signal-generation itself is expensive, use Broadcaster instead 
 * If keeping a ReadOnlyMutable around is fine, use that instead
 * Otherwise, there's this for the sake of convenience :)
 * or needing to keep a separate ReadOnlyMutable around
 *
 * Example:
 * pub struct Foo {
 *   pub active_signal: BoxSignalFn<bool>,
 * }
 *
 *
 * impl Foo {
 *   pub fn new<S: Signal<Item = bool> + 'static>(active_signal: impl Fn() -> S + 'static) -> Self {
 *     Self {
 *       active_signal: box_signal_fn(active_signal),
 *    }
 * }
 *
 * let foo = Foo::new(|| always(true));
 */

/// Type alias for a boxed signal-factory
pub type BoxSignalFn<T> = Box<dyn Fn() -> Pin<Box<dyn Signal<Item = T>>>>;

/// Helper to create boxed signal-factories
pub fn box_signal_fn<T, S: Signal<Item = T> + 'static>(f: impl Fn() -> S + 'static) -> BoxSignalFn<T> {
    Box::new(move || {
        Box::pin(f())
    })
}

/// Type alias for a Rc'd signal-factory (same idea as BoxSignalFn but cloneable)
pub type RcSignalFn<T> = Rc<dyn Fn() -> Pin<Box<dyn Signal<Item = T>>>>;

/// Helper to create Rc signal-factories (same idea as box_signal_fn but cloneable)
pub fn rc_signal_fn<T, S: Signal<Item = T> + 'static>(f: impl Fn() -> S + 'static) -> RcSignalFn<T> {
    Rc::new(move || {
        Box::pin(f())
    })
}
/*
 * These all generally solving the problem of where you need to return different types of signals
 * But don't want to Box it.
 *
 * Most of the helpers are for cases where the inner type of the signals are the same
 * And where it's usually about returning an always() vs. custom signal
 *
 * Fwiw, Boxing signals looks like this, for example:
    match foo {
        None =>
            Box::pin(always(None)) as Pin<Box<dyn Signal<Item = Option<Dom>>>>,
        Some(bar) =>
            Box::pin(get_some_signal(bar)) as Pin<Box<dyn Signal<Item = Option<Dom>>>>
    }
*/

/* TIPS FOR IMPLEMENTATION
 * a Signal must always return Poll::Ready(Some(...)) the first time it is called
 * after that it can return either Poll::Ready(Some(...)), Poll::Pending, or Poll::Ready(None)
 * and if it returns Poll::Ready(None), then from that point forward it's 
 * considered finished, the consumer must not poll again.
*/

use futures_signals::signal::{Signal, SignalExt};
use std::marker::Unpin;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::rc::Rc;

/// If the provided signal is None,
/// then a signal of the provided default
/// otherwise, the signal's value
pub struct DefaultSignal<S, T>
where
    S: Signal<Item = T>,
{
    default: Option<T>,
    value_signal: Option<S>,
    const_has_fired: bool,
}

impl<S, T> DefaultSignal<S, T>
where
    S: Signal<Item = T>,
{
    pub fn new(default: T, value_signal: Option<S>) -> Self {
        Self {
            default: Some(default),
            value_signal,
            const_has_fired: false,
        }
    }
}

impl<S, T> Signal for DefaultSignal<S, T>
where
    S: Signal<Item = T> + Unpin,
    T: Unpin,
{
    type Item = T;

    fn poll_change(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let _self = self.get_mut();

        match &mut _self.value_signal {
            None => {
                if _self.const_has_fired {
                    Poll::Ready(None)
                } else {
                    _self.const_has_fired = true;
                    Poll::Ready(_self.default.take())
                }
            }
            Some(value_signal) => value_signal.poll_change_unpin(cx),
        }
    }
}

/// If the provided signal is None,
/// then a signal of None
/// otherwise, a signal of Some(value)
pub struct OptionSignal<S, T>
where
    S: Signal<Item = T>,
{
    value_signal: Option<S>,
    const_has_fired: bool,
}

impl<S, T> OptionSignal<S, T>
where
    S: Signal<Item = T>,
{
    pub fn new(value_signal: Option<S>) -> Self {
        Self {
            value_signal,
            const_has_fired: false,
        }
    }
}

impl<S, T> Signal for OptionSignal<S, T>
where
    S: Signal<Item = T> + Unpin,
    T: Unpin,
{
    type Item = Option<T>;

    fn poll_change(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let _self = self.get_mut();

        match &mut _self.value_signal {
            None => {
                if _self.const_has_fired {
                    Poll::Ready(None)
                } else {
                    _self.const_has_fired = true;
                    Poll::Ready(Some(None))
                }
            }
            Some(value_signal) => {
                value_signal
                    .poll_change_unpin(cx)
                    //need to map the inner Option
                    //outer one is just Poll
                    .map(|value| value.map(|value| Some(value)))
            }
        }
    }
}

// A signal of either left or right
pub enum EitherSignal<Left, Right> {
    Left(Left),
    Right(Right),
}

impl<Left, Right, T> Signal for EitherSignal<Left, Right>
where
    Left: Signal<Item = T> + Unpin,
    Right: Signal<Item = T> + Unpin,
{
    type Item = T;

    fn poll_change(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        match self.get_mut() {
            Self::Left(x) => x.poll_change_unpin(cx),
            Self::Right(x) => x.poll_change_unpin(cx),
        }
    }
}
