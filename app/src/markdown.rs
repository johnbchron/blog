use pulldown_cmark::{CowStr, Event};

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

pub fn markdown_to_html(markdown: &str) -> String {
  let parser =
    pulldown_cmark::Parser::new_ext(markdown, pulldown_cmark::Options::all());
  let events = add_markdown_heading_ids(parser.into_iter().collect());
  let events = highlight_pulldown::highlight_with_theme(
    events.into_iter(),
    "base16-ocean.dark",
  )
  .unwrap();
  let mut html_output = String::new();
  pulldown_cmark::html::push_html(&mut html_output, events.into_iter());

  html_output
}
