mod highlighter;

use pulldown_cmark::{Options, Parser};

use self::highlighter::Highlighter;

pub(crate) struct Markdown<'a>(&'a str);

impl<'a> Markdown<'a> {
  pub fn new(input: &'a str) -> Self { Self(input) }

  pub fn render_to_html(self) -> String {
    let Self(input) = self;

    let mut html_out = String::new();
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);
    // opts.insert(Options::ENABLE_SMART_PUNCTUATION);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    opts.insert(Options::ENABLE_GFM);
    opts.insert(Options::ENABLE_SUPERSCRIPT);
    opts.insert(Options::ENABLE_SUBSCRIPT);

    let parser = Parser::new_ext(input, opts);
    let events = parser.collect::<Vec<_>>();
    let highlighted_events =
      Highlighter::highlight(events.into_iter()).expect("failed to highlight");

    pulldown_cmark::html::push_html(
      &mut html_out,
      highlighted_events.into_iter(),
    );

    html_out
  }
}
