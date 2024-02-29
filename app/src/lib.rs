use std::io::Read;

use gray_matter::{engine::TOML, Matter};
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

// fn highlight_code(code: &str, lang: &str) -> String {
//   use syntect::{
//     easy::HighlightLines,
//     highlighting::{Style, ThemeSet},
//     parsing::SyntaxSet,
//     util::{as_24_bit_terminal_escaped, LinesWithEndings},
//   };

//   let ps = SyntaxSet::load_defaults_newlines();
//   let ts = ThemeSet::load_defaults();

//   println!("lang: {}", lang);
//   let syntax = ps
//     .find_syntax_by_extension(lang.split('.').last().unwrap_or("txt"))
//     .unwrap();

//   let output = syntect::html::highlighted_html_for_string(
//     code,
//     &ps,
//     &syntax,
//     &ts.themes["base16-ocean.dark"],
//   );

//   output.unwrap()
// }

// fn add_hightlighting(events: Vec<Event<'_>>) -> Vec<Event<'_>> {
//   let mut parsing_code_block = false;
//   let mut code_block_language = String::new();
//   let mut code_block_content = String::new();
//   let mut events_to_return = Vec::new();

//   for event in events {
//     match event {
//       Event::Start(pulldown_cmark::Tag::CodeBlock(
//         pulldown_cmark::CodeBlockKind::Fenced(ref lang),
//       )) => {
//         parsing_code_block = true;
//         code_block_language = lang.to_string();
//         code_block_content.clear();
//       }
//       Event::End(pulldown_cmark::Tag::CodeBlock(
//         pulldown_cmark::CodeBlockKind::Fenced(_),
//       )) => {
//         parsing_code_block = false;
//         let highlighted_code =
//           highlight_code(&code_block_content, &code_block_language);
//         events_to_return.push(Event::Html(CowStr::from(highlighted_code)));
//       }
//       Event::Text(ref text) => {
//         if parsing_code_block {
//           code_block_content.push_str(text);
//           continue;
//         }
//       }
//       _ => {}
//     };
//     events_to_return.push(event);
//   }

//   events_to_return
// }

fn get_markdown_content(path: String) -> String {
  let path = format!("./content/{path}");
  let mut file = std::fs::File::open(&path).expect("failed to open file");
  let mut input = String::new();
  file
    .read_to_string(&mut input)
    .expect("failed to read file");

  let matter = Matter::<TOML>::new().parse(&input);

  let parser = pulldown_cmark::Parser::new_ext(
    &matter.content,
    pulldown_cmark::Options::all(),
  );
  let events = add_markdown_heading_ids(parser.into_iter().collect());
  // let events = add_hightlighting(events);
  let events = highlight_pulldown::highlight_with_theme(
    events.into_iter(),
    "base16-ocean.dark",
  )
  .unwrap();
  let mut html_output = String::new();
  pulldown_cmark::html::push_html(&mut html_output, events.into_iter());

  html_output
}

#[component]
fn Markdown(
  #[prop(into)] path: String,
  #[prop(into, default = String::new())] class: String,
) -> impl IntoView {
  let content = get_markdown_content(path);
  view! {
    <div class=format!("markdown {class}")>{html::div().inner_html(content)}</div>
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
