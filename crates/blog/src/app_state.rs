use std::{
  collections::HashMap,
  io::Read,
  path::{Path, PathBuf},
};

use miette::{Context, IntoDiagnostic};

use crate::posts::{Post, load_posts};

#[derive(Debug, Clone)]
pub(crate) struct AppState {
  static_asset_dir: PathBuf,
  stylesheet:       String,
  posts:            HashMap<String, Post>,
}

impl AppState {
  pub async fn build() -> miette::Result<Self> {
    let static_asset_dir = std::env::var("STATIC_ASSET_DIR")
      .into_diagnostic()
      .context("`STATIC_ASSET_DIR` env var not populated")?;
    let static_asset_dir = PathBuf::from(static_asset_dir);

    let posts_dir = std::env::var("POSTS_DIR")
      .map(PathBuf::from)
      .into_diagnostic()
      .context("`POSTS_DIR` env var not populated")?;

    let posts = load_posts(&posts_dir);

    let stylesheet_path = std::env::var("STYLESHEET_PATH")
      .into_diagnostic()
      .context("`STYLESHEET_PATH` env var not populated")?;
    let mut stylesheet_content = String::new();
    match std::fs::File::open(&stylesheet_path) {
      Ok(mut f) => {
        f.read_to_string(&mut stylesheet_content)
          .into_diagnostic()
          .context("failed to read from stylesheet file")?;
      }
      Err(e) => {
        tracing::warn!("failed to open stylesheet file: {e}");
      }
    };

    Ok(AppState {
      static_asset_dir,
      stylesheet: stylesheet_content,
      posts,
    })
  }

  pub fn static_asset_dir(&self) -> &Path { &self.static_asset_dir }

  pub fn stylesheet(&self) -> &str { &self.stylesheet }

  pub fn get_post(&self, slug: &str) -> Option<&Post> { self.posts.get(slug) }

  pub fn iter_posts(&self) -> impl Iterator<Item = (&String, &Post)> {
    self.posts.iter()
  }
}
