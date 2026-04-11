use axum::response::IntoResponse;

use crate::{ctx::ResponseSeed, page_wrapper::page_wrapper};

pub(crate) async fn test_page(
  ResponseSeed(ctx, resp): ResponseSeed,
) -> impl IntoResponse {
  const TITLE: &str = "The Hitchhiker's Guide to Markdown";
  let page = maud::PreEscaped(include_str!("./test_markup.html").to_owned());

  let doc = page_wrapper(TITLE, page, ctx);
  resp.into_stream(doc)
}
