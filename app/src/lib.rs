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
        <main class="mx-auto max-w-2xl pt-6 text-[#f5f5f5]">
          <Routes>
            <Route path="" view=HomePage/>
          </Routes>
        </main>
      </Router>
    </div>
  }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
  view! {
    <p class="text-4xl font-semibold tracking-tight">"Welcome to Leptos!"</p>
  }
}
