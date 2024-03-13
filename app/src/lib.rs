#[cfg(feature = "ssr")]
mod markdown;
mod posts;

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
      <leptos_meta::Link
        rel="preload" href="/fonts/Firava.woff2"
        as_="font" type_="font/woff2" crossorigin="anonymous"
      />
      <leptos_meta::Link rel="preload" href="/fonts/IosevkaTerm-Regular.woff2" as_="font" type_="font/woff2" crossorigin="anonymous" />

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
            <Route path="post/:path" view=posts::PostPage />
          </Routes>
        </div>
      </Router>
    </div>
  }
}

/// A styled hyperlink.
#[component]
fn Link(
  #[prop(into, default = String::new())] class: String,
  href: &'static str,
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
  let posts_resource =
    create_blocking_resource(|| (), |_| posts::get_all_posts());

  view! {
    <Suspense>
      { move || posts_resource.get().map(|p| match p {
        Ok(posts) => view! {
          <div class="flex flex-col gap-4">
            { posts.iter().map(|p| view! { <p>{p.metadata.title.clone()}</p> }).collect_view() }
          </div>
        }.into_view(),
        _ => view! { <p>"Loading..."</p> }.into_view()
      })}
    </Suspense>
  }
}
