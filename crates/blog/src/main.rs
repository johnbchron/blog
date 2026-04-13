mod app_state;
mod ctx;
mod home_page;
mod page_wrapper;
mod post_page;
mod posts;
mod setup_tracing;
mod signals;
mod test_page;

use axum::{
  Router, ServiceExt, handler::Handler, response::IntoResponse, routing::get,
};
use maud::html;
use miette::{Context, IntoDiagnostic};
use tower::ServiceBuilder;
use tower_http::{
  ServiceBuilderExt, compression::CompressionLayer,
  normalize_path::NormalizePathLayer, services::ServeDir,
};

use self::{
  app_state::AppState, ctx::ResponseSeed, page_wrapper::page_wrapper,
  signals::shutdown_signal,
};

#[tokio::main]
async fn main() -> miette::Result<()> {
  self::setup_tracing::setup_tracing().context("failed to setup tracing")?;

  let app_state = AppState::build()
    .await
    .context("failed to build app state")?;

  let router = router().with_state(app_state.clone()).fallback_service(
    ServeDir::new(&app_state.static_asset_dir)
      .not_found_service(fallback.with_state(app_state.clone())),
  );

  let service = ServiceBuilder::new()
    // unify types
    .map_request_body(axum::body::Body::new)
    .map_response_body(axum::body::Body::new)
    // normalize paths and routing
    .layer(NormalizePathLayer::trim_trailing_slash())
    .layer(CompressionLayer::new())
    // turn into a service
    .service(router)
    .into_make_service();

  let addr = "[::]:3000";
  let listener = tokio::net::TcpListener::bind(&addr)
    .await
    .into_diagnostic()
    .context(format!("failed to bind listener to `{addr}`"))?;
  let addr = listener.local_addr().into_diagnostic().with_context(|| {
    format!("failed to read local_addr of listener (requested {addr:?})")
  })?;
  tracing::info!("bound to http://{addr}");

  axum::serve(listener, service)
    .with_graceful_shutdown(shutdown_signal())
    .await
    .expect("failed to serve axum server");

  Ok(())
}

async fn fallback(ResponseSeed(ctx, resp): ResponseSeed) -> impl IntoResponse {
  const TITLE: &str = "That page doesn't exist.";
  let page = html! {
    a class="link" href="/" { "Go home" }
  };

  let doc = page_wrapper(TITLE, page, ctx);
  resp.into_stream(doc)
}

fn router() -> Router<AppState> {
  Router::new()
    .route("/", get(self::home_page::home_page))
    .route("/posts", get(self::post_page::all_posts_page))
    .route("/posts/{slug}", get(self::post_page::post_page))
    .route("/test", get(self::test_page::test_page))
}
