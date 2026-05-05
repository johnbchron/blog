use atom_syndication::{
  Content, Entry, Feed, FixedDateTime, Link, Person, Text,
};
use axum::{extract::State, http::header, response::IntoResponse};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Utc};

use crate::{app_state::AppState, home_page::SITE_DESCRIPTION, posts::Post};

const JOHN_LEWIS_URI: &str = "https://jlewis.sh/";
const BLOG_URI: &str = "https://blog.jlewis.sh/";
const BLOG_ATOM_FEED_ABSOLUTE_URI: &str = "https://blog.jlewis.sh/atom.xml";
const FAVICON_URI: &str = "https://blog.jlewis.sh/favicon.svg";

// assigns everything to happen at 8 AM UTC.
pub fn post_date_to_fixed_date_time(date: NaiveDate) -> FixedDateTime {
  NaiveDateTime::new(date, NaiveTime::from_hms_opt(8, 0, 0).unwrap())
    .and_local_timezone(Utc)
    .unwrap()
    .into()
}

pub fn build_feed<'a, I, S>(post_iter: I) -> Feed
where
  I: Iterator<Item = (&'a S, &'a Post)>,
  S: AsRef<str> + 'a,
{
  let posts = post_iter
    .map(|(s, p)| (s.to_owned(), p.clone()))
    .collect::<Vec<_>>();

  let last_updated = post_date_to_fixed_date_time(
    posts.iter().map(|(_, p)| p.date).max().unwrap_or_default(),
  );

  let john_lewis_person = Person {
    name:  "John Lewis".to_owned(),
    email: Some("contact@jlewis.sh".to_owned()),
    uri:   Some(JOHN_LEWIS_URI.to_string()),
  };

  let entries = posts.into_iter().map(|(s, p)| Entry {
    title:        Text::plain(&*p.title),
    id:           s.as_ref().to_string(),
    updated:      post_date_to_fixed_date_time(p.date),
    authors:      vec![john_lewis_person.clone()],
    // TODO: populate categories
    categories:   vec![],
    contributors: vec![john_lewis_person.clone()],
    links:        vec![Link {
      href:      format!("{BLOG_URI}posts/{slug}", slug = s.as_ref()),
      hreflang:  None,
      rel:       "self".to_string(),
      mime_type: Some("text/html".to_string()),
      length:    None,
      title:     Some(p.title.to_string()),
    }],
    published:    Some(post_date_to_fixed_date_time(p.date)),
    rights:       None,
    source:       None,
    summary:      None,
    content:      Some(Content {
      base:         Some(BLOG_URI.to_string()),
      lang:         None,
      value:        Some(p.body.to_string()),
      src:          None,
      content_type: Some("text/html".to_string()),
    }),
    extensions:   Default::default(),
  });

  let feed = Feed {
    title:        Text::plain("John Lewis"),
    id:           BLOG_ATOM_FEED_ABSOLUTE_URI.to_string(),
    updated:      last_updated,
    authors:      vec![john_lewis_person.clone()],
    // TODO: assign categories to the blog
    categories:   vec![],
    contributors: vec![john_lewis_person.clone()],
    generator:    None,
    icon:         Some(FAVICON_URI.to_string()),
    links:        vec![Link {
      href:      BLOG_URI.to_string(),
      rel:       "self".to_string(),
      hreflang:  None,
      mime_type: Some("text/html".to_string()),
      title:     Some("John Lewis".to_string()),
      length:    None,
    }],
    logo:         None,
    rights:       None,
    subtitle:     Some(Text::plain(SITE_DESCRIPTION)),
    entries:      entries.collect(),
    extensions:   Default::default(),
    namespaces:   Default::default(),
    base:         Some(BLOG_URI.to_string()),
    lang:         None,
  };

  tracing::info!("built feed with {} entries", feed.entries.len());

  feed
}

pub(crate) async fn feed_xml(
  State(state): State<AppState>,
) -> impl IntoResponse {
  (
    [(
      header::CONTENT_TYPE,
      header::HeaderValue::from_static("application/atom+xml"),
    )],
    state.feed_str().to_owned(),
  )
}
