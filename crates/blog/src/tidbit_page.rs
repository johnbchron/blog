use axum::{
  extract::{Path, State},
  http::StatusCode,
  response::IntoResponse,
};
use maud::{PreEscaped, html};

use crate::{
  app_state::AppState, ctx::ResponseSeed, page_wrapper::page_wrapper,
};

pub(crate) async fn tidbit_page(
  Path(slug): Path<String>,
  State(state): State<AppState>,
  ResponseSeed(ctx, resp): ResponseSeed,
) -> Result<impl IntoResponse, StatusCode> {
  let post = state
    .get_tidbit(slug.as_str())
    .ok_or(StatusCode::NOT_FOUND)?;

  let date = &post.date;
  let body_html = PreEscaped(&*post.body);
  let page = html! {
      small class="text-sm text-light-fg-dim dark:text-dark-fg-dim" {
        time datetime=(date) { (date) }
      }
      (body_html)
  };

  let doc = page_wrapper(&*post.title, page, ctx);
  Ok(resp.into_stream(doc))
}

pub(crate) async fn all_tidbits_page(
  State(state): State<AppState>,
  ResponseSeed(ctx, resp): ResponseSeed,
) -> impl IntoResponse {
  const DATE_CLASS: &str = "text-light-fg-dim dark:text-dark-fg-dim";

  let mut posts = state
    .iter_tidbits()
    .map(|(t, p)| (t, (&p.title, &p.date)))
    .collect::<Vec<_>>();
  // sort first descending by date, and then reverse the list
  posts.sort_unstable_by_key(|(_, (_, d))| *d);
  posts.reverse();

  let page = html! {
    ul {
      @for (slug, (title, date)) in posts {
        li {
          p {
            a class="link" href=(format!("/tidbits/{slug}")) { (title) }
            " – "
            time class=(DATE_CLASS) datetime=(date) { (date) }
          }
        }
      }
    }
  };

  let doc = page_wrapper("Tidbits", page, ctx);
  resp.into_stream(doc)
}
