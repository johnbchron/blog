use app::*;
use axum::{routing::post, Router};
use fileserv::file_and_error_handler;
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use tower_http::compression::CompressionLayer;

pub mod fileserv;

#[tokio::main]
async fn main() {
  simple_logger::init_with_level(log::Level::Info)
    .expect("couldn't initialize logging");

  let conf = get_configuration(None).await.unwrap();
  let leptos_options = conf.leptos_options;
  let addr = leptos_options.site_addr;
  let routes = generate_route_list(App);

  let app = Router::new()
    .route("/api/*fn_name", post(leptos_axum::handle_server_fns))
    .leptos_routes(&leptos_options, routes, App)
    .fallback(file_and_error_handler)
    .layer(CompressionLayer::new())
    .with_state(leptos_options);

  log::info!("listening on http://{}", &addr);
  axum::serve(tokio::net::TcpListener::bind(&addr).await.unwrap(), app)
    .await
    .unwrap();
}
