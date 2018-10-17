extern crate cfg_if;
extern crate futures;
extern crate js_sys;
extern crate wasm_bindgen;
extern crate wasm_bindgen_futures;
extern crate web_sys;
extern crate jieba_rs;

mod utils;

use cfg_if::cfg_if;
use futures::{Async, Future, Poll};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;
use wasm_bindgen_futures::JsFuture;

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}


/// A future that becomes ready after a tick of the micro task queue.
pub struct NextTick {
    inner: JsFuture,
}

impl NextTick {
    /// Construct a new `NextTick` future.
    pub fn new() -> NextTick {
        // Create a resolved promise that will run its callbacks on the next
        // tick of the micro task queue.
        let promise = js_sys::Promise::resolve(&JsValue::NULL);
        // Convert the promise into a `JsFuture`.
        let inner = JsFuture::from(promise);
        NextTick { inner }
    }
}

impl Future for NextTick {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<(), ()> {
        // Polling a `NextTick` just forwards to polling if the inner promise is
        // ready.
        match self.inner.poll() {
            Ok(Async::Ready(_)) => Ok(Async::Ready(())),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(_) => unreachable!(
                "We only create NextTick with a resolved inner promise, never \
                 a rejected one, so we can't get an error here"
            ),
        }
    }
}

#[wasm_bindgen]
pub struct Jieba {
    instance: jieba_rs::Jieba
}

#[wasm_bindgen]
impl Jieba {
    #[wasm_bindgen]
    pub fn new () -> Jieba {
        Jieba {
            instance: jieba_rs::Jieba::new()
        }
    }

    #[wasm_bindgen]
    pub fn load_dict(&mut self, in_dict: &mut [u8]) -> js_sys::Promise {
        let dict: &[u8] = &*in_dict;
        let future = futures::future::ok::<(), ()>(())
            .and_then(move |_| {
                self.instance.load_dict(&mut &*dict);
                Ok(())
            })
            .map(|_| JsValue::TRUE)
            .map_err(|error| {
                let js_error = js_sys::Error::new(&format!("uh oh! {:?}", error));
                JsValue::from(js_error)
            });
        future_to_promise(future)
    }

    #[wasm_bindgen]
    pub fn cut(&self, sentence: String, hmm: Option<bool>) -> Box<[JsValue]> {
        let words = self.instance.cut(&sentence, hmm.unwrap_or(false));
        words
            .iter()
            .map(|&x| JsValue::from(x))
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }
}

