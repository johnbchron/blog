use axum::response::IntoResponse;
use maud::html;

pub const SITE_DESCRIPTION: &str = "Welcome to my space. Please enjoy a \
                                    select sampling of my thoughts, lossily \
                                    translated into words. My opinions are my \
                                    own, and all content is of human origin.";

use crate::{ctx::ResponseSeed, page_wrapper::page_wrapper};

pub(crate) async fn home_page(
  ResponseSeed(ctx, resp): ResponseSeed,
) -> impl IntoResponse {
  const TITLE: &str = "John Lewis";
  let page = html! {
    p { (SITE_DESCRIPTION) }

    p {
      "View "
      a href="/posts" { "my blog posts" }
      "."
    }
  };

  let doc = page_wrapper(TITLE, page, ctx);
  resp.into_stream(doc)
}
