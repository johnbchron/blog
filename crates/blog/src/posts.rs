use std::{collections::HashMap, path::PathBuf, sync::Arc};

use serde::Deserialize;

use crate::markdown::Markdown;

#[derive(Debug, Clone)]
pub(crate) struct Post {
  pub(crate) title: Arc<str>,
  pub(crate) date:  Arc<str>,
  pub(crate) body:  Arc<str>,
}

#[derive(Debug, Deserialize)]
struct Frontmatter {
  title: String,
  date:  String,
}

#[derive(Debug)]
pub(crate) enum PostError {
  MissingOpenDelimiter,
  MissingCloseDelimiter,
  Frontmatter(toml::de::Error),
}

impl std::fmt::Display for PostError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::MissingOpenDelimiter => write!(f, "missing opening +++ delimiter"),
      Self::MissingCloseDelimiter => write!(f, "missing closing +++ delimiter"),
      Self::Frontmatter(e) => write!(f, "frontmatter parse error: {e}"),
    }
  }
}

impl From<toml::de::Error> for PostError {
  fn from(e: toml::de::Error) -> Self { Self::Frontmatter(e) }
}

impl Post {
  pub(crate) fn from_markdown(content: &str) -> Result<Self, PostError> {
    let content = content.strip_prefix('\u{feff}').unwrap_or(content);
    let rest = content
      .strip_prefix("+++")
      .ok_or(PostError::MissingOpenDelimiter)?;
    let (fm, body) = rest
      .split_once("+++")
      .ok_or(PostError::MissingCloseDelimiter)?;
    let fm: Frontmatter = toml::from_str(fm.trim())?;
    let body = body.strip_prefix('\n').unwrap_or(body);

    let html = Markdown::new(body).render_to_html();

    Ok(Post {
      title: fm.title.into(),
      date:  fm.date.into(),
      body:  html.into(),
    })
  }
}

pub(crate) fn load_posts(dir: &PathBuf) -> HashMap<Arc<str>, Post> {
  let mut posts = HashMap::new();
  let Ok(entries) = std::fs::read_dir(dir) else {
    tracing::warn!("posts directory not found: {}", dir.display());
    return posts;
  };

  for entry in entries.flatten() {
    let path = entry.path();
    if path.extension().is_some_and(|ext| ext == "md") {
      let slug = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();

      match std::fs::read_to_string(&path) {
        Ok(content) => match Post::from_markdown(&content) {
          Ok(post) => {
            tracing::info!("loaded post: {slug}");
            posts.insert(slug.into(), post);
          }
          Err(e) => {
            tracing::warn!("skipping {}: {e}", path.display());
          }
        },
        Err(e) => {
          tracing::warn!("skipping {}: read error: {e}", path.display());
        }
      }
    }
  }

  posts
}
