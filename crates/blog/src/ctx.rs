use std::sync::Arc;

use axum::{
  extract::{FromRef, FromRequestParts},
  http::request::Parts,
};
use columbo::{Html, SuspendedResponse, SuspenseContext};

use crate::app_state::AppState;

pub struct Ctx(Arc<CtxInner>);

impl Clone for Ctx {
  fn clone(&self) -> Self { Self(self.0.clone()) }
}

struct CtxInner {
  app_state:    AppState,
  suspense_ctx: SuspenseContext,
}

impl Ctx {
  /// Returns the [`AppState`].
  pub fn state(&self) -> &AppState { &self.0.app_state }

  /// Suspends a future with columbo.
  pub fn suspend<F, Fut, M>(
    &self,
    f: F,
    placeholder: impl Into<Html>,
  ) -> columbo::Suspense
  where
    F: FnOnce(Ctx) -> Fut,
    Fut: Future<Output = M> + Send + 'static,
    M: Into<Html> + 'static,
  {
    self.0.suspense_ctx.suspend(f(self.clone()), placeholder)
  }
}

/// The extractor that begins every page. Automatically starts columbo context.
pub struct ResponseSeed(pub Ctx, pub SuspendedResponse);

impl<S> FromRequestParts<S> for ResponseSeed
where
  S: Send + Sync,
  AppState: FromRef<S>,
{
  type Rejection = ();

  async fn from_request_parts(
    _parts: &mut Parts,
    state: &S,
  ) -> Result<Self, Self::Rejection> {
    let (suspense_ctx, resp) =
      columbo::new_with_opts(columbo::ColumboOptions {
        panic_renderer: None,
        auto_cancel:    None,
        include_script: Some(false),
      });

    let app_state = AppState::from_ref(state);

    let ctx = Ctx(Arc::new(CtxInner {
      app_state,
      suspense_ctx,
    }));

    Ok(ResponseSeed(ctx, resp))
  }
}
