use axum::response::IntoResponse;
use maud::html;

use crate::{ctx::ResponseSeed, page_wrapper::page_wrapper};

pub(crate) async fn about_page(
  ResponseSeed(ctx, resp): ResponseSeed,
) -> impl IntoResponse {
  const TITLE: &str = "About";

  let page = html! {
    p { "My name is John Lewis, and I'm a software developer. I work primarily in Rust, and I love things that are truly innovative. I have a driving desire to evaluate things from first principles and to conduct my work with care." }

    p { "I was born and raised in Texas, and spent some time living in Switzerland." }

    p { "I love Jesus and owe my life to him. He is my everything. He is the reason I strive to build with excellence." }

    hr;

    p {
      "All the writing on this blog is strictly of human origin. I will not publish AI-generated content I believe it generally exacerbates the effect of "
      a href="https://en.wikipedia.org/wiki/Brandolini's_law" { "Brandolini's law" }
      ", and I generally don't like reading it myself."
    }

    p {
      "If you have a comment to make or question to ask about my work, or any other inquiry, please reach out to me at "
      a href="mailto:contact@jlewis.sh" { "contact@jlewis.sh" }
      "."
    }
  };

  let doc = page_wrapper(TITLE, page, ctx);
  resp.into_stream(doc)
}
