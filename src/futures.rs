use discard::DiscardOnDrop;
use futures_signals::{
    cancelable_future, 
    CancelableFutureHandle, 
    signal::{Signal, Mutable}
};
use wasm_bindgen_futures::spawn_local;
use std::{
    future::Future,
    sync::atomic::{AtomicUsize, Ordering}
};
use futures::future::{abortable, AbortHandle};


/// Makes it easier to run a future in the background with the ability to cancel and/or swap it
/// The status of the most-recent future can be gotten as a signal via `is_loading()`
pub struct AsyncLoader {
    loading: Mutable<Option<AsyncState>>,
}

impl Drop for AsyncLoader {
    fn drop(&mut self) {
        self.cancel();
    }
}

impl AsyncLoader {
    pub fn new() -> Self {
        Self {
            loading: Mutable::new(None),
        }
    }

    pub fn cancel(&self) {
        self.replace(None);
    }

    fn replace(&self, value: Option<AsyncState>) {
        let mut loading = self.loading.lock_mut();

        if let Some(state) = loading.as_mut() {
            state.handle.abort();
        }

        *loading = value;
    }

    pub fn load<F>(&self, fut: F) where F: Future<Output = ()> + 'static {
        let (fut, handle) = abortable(fut);

        let state = AsyncState::new(handle);
        let id = state.id;

        self.replace(Some(state));

        let loading = self.loading.clone();

        spawn_local(async move {
            match fut.await {
                Ok(()) => {
                    let mut loading = loading.lock_mut();

                    if let Some(current_id) = loading.as_ref().map(|x| x.id) {
                        // If it hasn't been overwritten with a new state...
                        if current_id == id {
                            *loading = None;
                        }
                    }
                },
                // It was already cancelled
                Err(_) => {},
            }
        });
    }

    pub fn is_loading(&self) -> impl Signal<Item = bool> {
        self.loading.signal_ref(|x| x.is_some())
    }
}
struct AsyncState {
    id: usize,
    handle: AbortHandle,
}

impl AsyncState {
    fn new(handle: AbortHandle) -> Self {
        static ID: AtomicUsize = AtomicUsize::new(0);

        let id = ID.fetch_add(1, Ordering::SeqCst);

        Self { id, handle }
    }
}

/// Makes it easier to spawn a future into a cancellable signal
#[inline]
pub fn spawn_future<F>(future: F) -> DiscardOnDrop<CancelableFutureHandle>
    where F: Future<Output = ()> + 'static {
    let (handle, future) = cancelable_future(future, || ());

    spawn_local(future);

    handle
}
