use axum::response::IntoResponse;
use maud::html;

use crate::{ctx::ResponseSeed, page_wrapper::page_wrapper};

pub(crate) async fn home_page(
  ResponseSeed(ctx, resp): ResponseSeed,
) -> impl IntoResponse {
  let page = html! {
    div class="flex flex-col gap-4" {
      p class="title" { "John Lewis" }
      p {
        "Welcome to my space. Please enjoy a select sampling of my thought-space, lossily translated into words. My opinions are my own, and all content is of human origin."
      }
    }
  };

  let doc = page_wrapper(page, ctx);
  resp.into_stream(doc)
}
