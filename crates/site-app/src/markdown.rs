use std::{io::Cursor, path::PathBuf, str::FromStr};

use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag};

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

fn highlight_code(events: Vec<Event<'_>>) -> Vec<Event<'_>> {
  use syntect::{
    highlighting::ThemeSet, html::highlighted_html_for_string,
    parsing::SyntaxSet,
  };

  let mut in_code_block = false;

  let syntax_set = SyntaxSet::load_defaults_nonewlines();
  let mut syntax = syntax_set.find_syntax_plain_text();

  // let theme_set = ThemeSet::load_defaults();
  // let theme = theme_set.themes.get("base16-ocean.dark").unwrap().clone();
  let theme = ThemeSet::load_from_reader(&mut Cursor::new(include_str!(
    "./rose-pine.tmTheme"
  )))
  .unwrap();

  let mut to_highlight = String::new();
  let mut out_events = Vec::new();

  for event in events {
    match event {
      Event::Start(Tag::CodeBlock(kind)) => {
        match kind {
          CodeBlockKind::Fenced(lang) => {
            syntax = syntax_set.find_syntax_by_token(&lang).unwrap_or(syntax)
          }
          CodeBlockKind::Indented => {}
        }
        in_code_block = true;
      }
      Event::End(Tag::CodeBlock(_)) => {
        if !in_code_block {
          panic!("this should never happen");
        }
        let html = highlighted_html_for_string(
          &to_highlight,
          &syntax_set,
          syntax,
          &theme,
        )
        .unwrap();

        to_highlight.clear();
        in_code_block = false;
        out_events.push(Event::Html(CowStr::from(html)));
      }
      Event::Text(t) => {
        if in_code_block {
          to_highlight.push_str(&t);
        } else {
          out_events.push(Event::Text(t));
        }
      }
      e => {
        out_events.push(e);
      }
    }
  }

  out_events
}

pub fn markdown_to_html(markdown: &str) -> String {
  let parser =
    pulldown_cmark::Parser::new_ext(markdown, pulldown_cmark::Options::all());
  let events = add_markdown_heading_ids(parser.into_iter().collect());
  let events = highlight_code(events);
  let mut html_output = String::new();
  pulldown_cmark::html::push_html(&mut html_output, events.into_iter());

  html_output
}
