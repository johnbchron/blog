use axum::{
  extract::Form,
  response::{Html, IntoResponse},
};
use maud::html;
use serde::Deserialize;

use crate::{
  ctx::ResponseSeed, markdown::Markdown, page_wrapper::page_wrapper,
};

const TEXTAREA_CLASS: &str = concat!(
  "w-full resize-y font-mono text-sm p-3 rounded-sm ",
  "border border-light-fg-dimmer/50 dark:border-dark-fg-dimmer/50 ",
  "bg-light-bg-2 dark:bg-dark-bg-2 ",
  "focus:outline-none focus:border-light-accent dark:focus:border-dark-accent",
);

const PLACEHOLDER: &str = "# Paste some markdown.";

pub(crate) async fn playground_page(
  ResponseSeed(ctx, resp): ResponseSeed,
) -> impl IntoResponse {
  const TITLE: &str = "Markdown Playground";

  let page = html! {
    p class="text-light-fg-dim dark:text-dark-fg-dim" {
      "Paste or type markdown below to see it rendered live."
    }
    p class="text-light-fg-dim dark:text-dark-fg-dim" {
      "I created this because I put a lot of energy into tuning the way this \
      blog renders markdown, and thus it is now my favorit markdown renderer."
    }

    textarea
      name="content"
      rows="5"
      spellcheck="false"
      placeholder=(PLACEHOLDER)
      class=(TEXTAREA_CLASS)
      hx-post="/playground/render"
      hx-trigger="input changed delay:100ms"
      hx-target="#preview"
      hx-swap="innerHTML" { }

    hr;

    div id="preview" class="markdown" { }
  };

  let doc = page_wrapper(TITLE, page, ctx);
  resp.into_stream(doc)
}

#[derive(Deserialize)]
pub(crate) struct RenderInput {
  content: String,
}

pub(crate) async fn render(
  Form(input): Form<RenderInput>,
) -> impl IntoResponse {
  Html(Markdown::new(&input.content).render_to_html())
}
