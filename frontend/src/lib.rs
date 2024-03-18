// This has to be imported for `wasm_bindgen` to work.
#[allow(unused_imports)]
use app::*;
use leptos::*;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn hydrate() {
  // initializes logging using the `log` crate
  _ = console_log::init_with_level(log::Level::Debug);
  console_error_panic_hook::set_once();

  leptos::leptos_dom::HydrationCtx::stop_hydrating();
}
