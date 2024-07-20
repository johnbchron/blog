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
      <Stylesheet href="/pkg/site.css"/>
      <Style>{include_str!("../style/iosevka_term.css")}</Style>

      // preloads the fonts
      <leptos_meta::Link
        rel="preload" href="/fonts/Firava.woff2"
        as_="font" type_="font/woff2" crossorigin="anonymous"
      />
      <leptos_meta::Link
        rel="preload" href="/fonts/Iosevkacustom-Regular.ttf"
        as_="font" type_="font/ttf" crossorigin="anonymous"
      />

      <leptos_meta::Link rel="icon" href="/favicon.png" type_="image/png" />

      // sets the document title
      <Title text="John Lewis' Blog" />

      <Router fallback=|| {
        let mut outside_errors = Errors::default();
        outside_errors.insert_with_default_key(AppError::NotFound);
        view! { <ErrorTemplate outside_errors/> }.into_view()
      }>
        <div class="px-4 md:px-0 md:mx-auto md:w-[48rem] py-4 text-neutral-100 text-lg">
          // header
          <div class="flex gap-2 w-full text-lg font-light">
            <StyledLink href="/">"John Lewis\' Blog"</StyledLink>
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
fn StyledLink(
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
  view! { <div class="h-[1px] w-full border-t-2 border-neutral-400/50 my-4" /> }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
  let posts_resource = create_resource(|| (), |_| posts::get_all_posts());

  let post_list_item = |p: posts::Post| {
    view! {
      <li>
        <a href={format!("/post/{}", p.path)}>
          {p.metadata.title}
        </a>
        " - " {p.metadata.written_on}
      </li>
    }
  };

  let post_elements = view! {
    <Suspense>
      { move || posts_resource.get().map(|p| match p {
        Ok(posts) => posts.clone().into_iter().map(post_list_item).collect_view(),
        _ => view! { <p>"This should never happen"</p> }.into_view()
      })}
    </Suspense>
  };

  view! {
    <div class="markdown">
      <h2>"Hey, John here!"</h2>
      <p>
        "Welcome to my blog. I write about my findings and thoughts, mostly regarding Rust, Nix, and game development. If you'd like to hire me, I'm available to hire! Contact me "
      <a href="mailto:contact@jlewis.sh">"here"</a>
      "."
      </p>
      <h3>"Recent Posts"</h3>
      <ul>
        {post_elements}
      </ul>
    </div>
  }
}
