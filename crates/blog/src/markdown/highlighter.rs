use std::sync::{LazyLock, Mutex};

use miette::{Context, IntoDiagnostic, Result};
use pulldown_cmark::{CodeBlockKind, Event, Tag, TagEnd};

static HIGHLIGHTER: LazyLock<Mutex<arborium::Highlighter>> =
  LazyLock::new(|| Mutex::new(arborium::Highlighter::new()));

pub struct Highlighter;

impl Highlighter {
  pub fn highlight<'a, It>(events: It) -> Result<Vec<Event<'a>>>
  where
    It: Iterator<Item = Event<'a>>,
  {
    let mut highlighter =
      HIGHLIGHTER.lock().expect("highlighter mutex is poisoned");

    let mut output_events = Vec::new();
    let mut code_buf: Option<(String, String)> = None; // (lang, code)

    for event in events {
      match event {
        Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang)))
          if !lang.is_empty() =>
        {
          code_buf = Some((lang.to_string(), String::new()));
        }
        Event::Text(text) => {
          if let Some((_, ref mut code)) = code_buf {
            code.push_str(&text);
          } else {
            output_events.push(Event::Text(text));
          }
        }
        ev @ Event::End(TagEnd::CodeBlock) => match code_buf.take() {
          Some((lang, code)) => {
            let highlighted = highlighter
              .highlight(&lang, &code)
              .into_diagnostic()
              .with_context(|| {
                format!("failed to highlight code with declared lang `{lang}`")
              })?;
            output_events.push(Event::Start(Tag::CodeBlock(
              CodeBlockKind::Fenced(lang.into()),
            )));
            output_events.push(Event::Html(highlighted.into()));
            output_events.push(Event::End(TagEnd::CodeBlock));
          }
          None => {
            output_events.push(ev);
          }
        },
        ev => {
          output_events.push(ev);
        }
      }
    }

    Ok(output_events)
  }
}
