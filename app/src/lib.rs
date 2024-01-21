use std::io::Read;

use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use pulldown_cmark::{CowStr, Event};

use crate::error_template::{AppError, ErrorTemplate};

pub mod error_template;

#[component]
pub fn App() -> impl IntoView {
  // Provides context that manages stylesheets, titles, meta tags, etc.
  provide_meta_context();

  view! {
    <div class="bg-neutral-800 min-h-screen">
      <Stylesheet id="leptos" href="/pkg/blog.css"/>
      <Stylesheet href="/fonts/iosevka_term/iosevka_term.css"/>

      // sets the document title
      <Title text="Welcome to Leptos"/>

      // content for this welcome page
      <Router fallback=|| {
        let mut outside_errors = Errors::default();
        outside_errors.insert_with_default_key(AppError::NotFound);
        view! { <ErrorTemplate outside_errors/> }.into_view()
      }>
        <main class="mx-auto max-w-3xl pt-4 text-neutral-100 text-lg">
          <Routes>
            <StaticRoute path="" view=HomePage static_params=|| Box::pin(async { StaticParamsMap::default() }) />
          </Routes>
        </main>
      </Router>
    </div>
  }
}

fn add_markdown_heading_ids(events: Vec<Event<'_>>) -> Vec<Event<'_>> {
  let mut parsing_header = false;
  let mut heading_id = String::new();
  let mut events_to_return = Vec::new();

  for event in events {
    match event {
      Event::Start(pulldown_cmark::Tag::Heading(_, _, _)) => {
        parsing_header = true;
        heading_id.clear();
      }
      Event::End(pulldown_cmark::Tag::Heading(_, _, _)) => {
        parsing_header = false;
        heading_id = slug::slugify(heading_id.as_str());

        events_to_return.push(Event::Text(CowStr::from(" ")));
        events_to_return.push(Event::Html(CowStr::from(format!(
          "<a href=\"#{}\" id=\"{}\"><span class=\"anchor-icon\">#</span></a>",
          heading_id, heading_id
        ))));
      }
      Event::Text(ref text) => {
        if parsing_header {
          heading_id.push_str(text);
        }
      }
      _ => {}
    }
    events_to_return.push(event);
  }

  events_to_return
}

pub async fn get_markdown_content(
  path: String,
) -> Result<String, ServerFnError> {
  let path = format!("./content/{path}");
  let mut file = std::fs::File::open(&path)?;
  let mut input = String::new();
  file.read_to_string(&mut input)?;

  let parser =
    pulldown_cmark::Parser::new_ext(&input, pulldown_cmark::Options::all());
  let events = add_markdown_heading_ids(parser.into_iter().collect());
  let mut html_output = String::new();
  pulldown_cmark::html::push_html(&mut html_output, events.into_iter());

  Ok(html_output)
}

#[component]
fn Markdown(
  #[prop(into)] path: String,
  #[prop(into, default = String::new())] class: String,
) -> impl IntoView {
  let content = create_resource(
    || (),
    move |_| {
      let path = path.clone();
      async move { get_markdown_content(path).await }
    },
  );

  view! {
    <Suspense fallback=move || view! { <p>"Loading (Suspense Fallback)..."</p> }>
      <div class=format!("markdown {class}")>{move || html::div().inner_html(content.get().map(|r| r.unwrap_or_default()).unwrap_or_default())}</div>
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
    <a class=format!("hover:underline {class}") href=href>{children()}</a>
  }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
  view! {
    <div class="flex flex-col">
      <div class="flex w-full text-lg">
        <Link class="items-center font-light" href="/">"John Lewis\' Blog"</Link>
        <div class="flex-1" />
        <p class="items-center font-light">"Rust, Games, Musings"</p>
      </div>
      <div class="h-[1px] w-full border-b border-[#f5f5f5]/50 my-4" />
      <Markdown path="homepage.md" />
    </div>
  }
}
