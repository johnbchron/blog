#[cfg(feature = "ssr")]
mod markdown;

use std::io::Read;

use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::error_template::{AppError, ErrorTemplate};

pub mod error_template;

#[component]
pub fn App() -> impl IntoView {
  // Provides context that manages stylesheets, titles, meta tags, etc.
  provide_meta_context();

  view! {
    <div class="bg-neutral-800 min-h-screen">
      <Stylesheet href="/pkg/blog.css"/>
      <Style>{include_str!("../../style/iosevka_term.css")}</Style>

      // sets the document title
      <Title text="Welcome to Leptos"/>

      // content for this welcome page
      <Router fallback=|| {
        let mut outside_errors = Errors::default();
        outside_errors.insert_with_default_key(AppError::NotFound);
        view! { <ErrorTemplate outside_errors/> }.into_view()
      }>
        <div class="px-4 md:px-0 md:mx-auto md:w-[48rem] pt-4 text-neutral-100 text-lg">
          // header
          <div class="flex gap-2 w-full text-lg font-light">
            <Link href="/">"John Lewis\' Blog"</Link>
            "|"
            <Link href="/posts">Posts</Link>
            <div class="flex-1" />
            <p class="items-center font-light">"Rust, Games, Musings"</p>
          </div>
          <Separator />
          <Routes>
            <Route path="" view=HomePage />
          </Routes>
        </div>
      </Router>
    </div>
  }
}

#[server]
async fn get_markdown_content(path: String) -> Result<String, ServerFnError> {
  #[cfg(feature = "ssr")]
  Ok(markdown::get_markdown_content(path))
}

#[component]
fn Markdown(
  #[prop(into)] path: String,
  #[prop(default = "")] class: &'static str,
) -> impl IntoView {
  let content =
    create_blocking_resource(move || path.clone(), get_markdown_content);

  view! {
    <Suspense>
      { move || content.get().map(|c| match c {
        Ok(ref content) => view!{
          <div class=format!("markdown {class}")>{html::div().inner_html(c.unwrap())}</div>
        }.into_view(),
        _ => {
          view! { <div>"Error loading content"</div> }.into_view()
        }
      })}
    </Suspense>
  }
}

/// A styled hyperlink.
#[component]
fn Link(
  #[prop(into, default = String::new())] class: String,
  #[prop(into)] href: String,
  children: Children,
) -> impl IntoView {
  view! {
    <a class=format!("text-periwinkle hover:underline {class}") href=href>{children()}</a>
  }
}

/// A full-width separator.
#[component]
fn Separator() -> impl IntoView {
  view! { <div class="h-[1px] w-full border-b border-neutral-100/50 my-4" /> }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
  view! {
    <Markdown path="posts/building-this-blog.md" />
  }
}
