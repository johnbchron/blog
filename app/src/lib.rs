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
    <div class="bg-[#252525] min-h-screen">
      <Stylesheet id="leptos" href="/pkg/blog.css"/>

      // sets the document title
      <Title text="Welcome to Leptos"/>
      <Script src="https://cdn.tailwindcss.com"/>

      // content for this welcome page
      <Router fallback=|| {
        let mut outside_errors = Errors::default();
        outside_errors.insert_with_default_key(AppError::NotFound);
        view! { <ErrorTemplate outside_errors/> }.into_view()
      }>
        <main class="mx-auto max-w-xl pt-4 text-[#f5f5f5]">
          <Routes>
            <Route path="" view=HomePage/>
          </Routes>
        </main>
      </Router>
    </div>
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
    <a class=format!("hover:underline {class}") href=href>{children()}</a>
  }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
  view! {
    <div class="flex flex-col gap-2">
      <div class="flex w-full text-lg">
        <Link class="items-center font-light" href="/">"John Lewis\' Blog"</Link>
        <div class="flex-1" />
        <p class="items-center font-light">"Rust, Games, Musings"</p>
      </div>
      <div class="h-[1px] w-full border-b border-[#f5f5f5]/50 mb-4" />
      <div>
        <p>"Hi! John here. I love building backend code and I\'m writing a technologically innovative magic-RPG game. Some other character traits of interest:"</p>
        <ul class="list-disc pl-6">
          <li>"I've been known to re-invent the wheel periodically"</li>
          <li>"I'm a pathological "<Link href="https://www.rust-lang.org/">"Rust"</Link>" evangelist."</li>
          <li>"I like walking; a lot."</li>
        </ul>
      </div>
    </div>
  }
}
