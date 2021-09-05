use std::{cell::RefCell, rc::Rc, task::Poll};

use futures::Future;

//
// Single threaded completable future
//

struct State<T> {
    waker: Option<std::task::Waker>,
    data: Option<T>,
}

pub struct FutureCompleter<T> {
    state: Rc<RefCell<State<T>>>,
}

impl<T> FutureCompleter<T> {
    pub fn new() -> (CompletableFuture<T>, FutureCompleter<T>) {
        let state = Rc::new(RefCell::new(State {
            waker: None,
            data: None,
        }));
        (
            CompletableFuture {
                state: state.clone(),
            },
            FutureCompleter { state },
        )
    }

    pub fn complete(self, data: T) {
        let mut state = self.state.borrow_mut();
        state.data.replace(data);
        if let Some(waker) = state.waker.take() {
            waker.wake();
        }
    }
}

pub struct CompletableFuture<T> {
    state: Rc<RefCell<State<T>>>,
}

impl<T> Future for CompletableFuture<T> {
    type Output = T;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.borrow_mut();
        let data = state.data.take();
        match data {
            Some(data) => Poll::Ready(data),
            None => {
                if state.waker.is_none() {
                    state.waker.replace(cx.waker().clone());
                }
                Poll::Pending
            }
        }
    }
}
