extern crate cfg_if;
extern crate futures;
extern crate js_sys;
extern crate wasm_bindgen;
extern crate wasm_bindgen_futures;
extern crate web_sys;
extern crate jieba_rs;
extern crate serde;
extern crate serde_json;

mod utils;

use cfg_if::cfg_if;
use futures::{Async, Future, Poll};
use futures::future::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;
use wasm_bindgen_futures::JsFuture;
use std::sync::Arc;
use std::sync::Mutex;
use std::result::Result::Ok;

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
    inner: Arc<Mutex<jieba_rs::Jieba>>,
}

#[wasm_bindgen]
impl Jieba {
    #[wasm_bindgen(constructor)]
    pub fn new () -> Jieba {
        Jieba {
            inner: Arc::new(Mutex::new(jieba_rs::Jieba::empty())),
        }
    }

    #[wasm_bindgen]
    pub fn load_dict(&mut self, dict: String) -> js_sys::Promise {
        let this = self.inner.clone();
        let future = NextTick::new()
            .and_then(move |_| {
                this
                    .lock()
                    .unwrap()
                    .load_dict(&mut dict.as_bytes());
                ok(())
            })
            .map(|_| JsValue::TRUE)
            .map_err(|error| {
                let js_error = js_sys::Error::new(&format!("uh oh! {:?}", error));
                JsValue::from(js_error)
            });
        future_to_promise(future)
    }

    #[wasm_bindgen]
    pub fn cut(&self, sentence: String, hmm: Option<bool>) -> js_sys::Promise {
        let this = self.inner.clone();
        let future = NextTick::new()
            .and_then(move |_| {
                let words = this
                    .lock()
                    .unwrap()
                    .cut(&sentence, hmm.unwrap_or(false));
                let parsed = JsValue::from_serde(&words);
                let res = match parsed {
                    serde::export::Err(error) => {
                        let js_error = js_sys::Error::new(&format!("uh oh! {:?}", error));
                        err(JsValue::from(js_error))
                    },
                    serde::export::Ok(val) => ok(val),
                };
                res
            });

        future_to_promise(future)
    }
}

