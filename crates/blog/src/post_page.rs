use axum::{
  extract::{Path, State},
  http::StatusCode,
  response::IntoResponse,
};
use maud::{PreEscaped, html};

use crate::{
  app_state::AppState, ctx::ResponseSeed, page_wrapper::page_wrapper,
};

pub(crate) async fn post_page(
  Path(slug): Path<String>,
  State(state): State<AppState>,
  ResponseSeed(ctx, resp): ResponseSeed,
) -> Result<impl IntoResponse, StatusCode> {
  let post = state.get_post(slug.as_str()).ok_or(StatusCode::NOT_FOUND)?;

  let date = &post.date;
  let body_html = PreEscaped(&*post.body);
  let page = html! {
      small class="text-sm text-light-fg-dim dark:text-dark-fg-dim" {
        time { (date) }
      }
      (body_html)
  };

  let doc = page_wrapper(&*post.title, page, ctx);
  Ok(resp.into_stream(doc))
}

pub(crate) async fn all_posts_page(
  State(state): State<AppState>,
  ResponseSeed(ctx, resp): ResponseSeed,
) -> impl IntoResponse {
  const LINK_CLASS: &str = "link text-light-accent dark:text-dark-accent";
  const DATE_CLASS: &str = "text-light-fg-dim dark:text-dark-fg-dim";

  let mut posts = state
    .iter_posts()
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
            a class=(LINK_CLASS) href=(format!("/posts/{slug}")) { (title) }
            " – "
            span class=(DATE_CLASS) { (date) }
          }
        }
      }
    }
  };

  let doc = page_wrapper("Posts", page, ctx);
  resp.into_stream(doc)
}
