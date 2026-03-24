use std::{io::Read, path::PathBuf, sync::Arc};

use miette::{Context, IntoDiagnostic};

#[derive(Debug, Clone)]
pub(crate) struct AppState {
  pub(crate) static_asset_dir: PathBuf,
  pub(crate) stylesheet:       Arc<str>,
}

impl AppState {
  pub async fn build() -> miette::Result<Self> {
    let static_asset_dir = std::env::var("STATIC_ASSET_DIR")
      .into_diagnostic()
      .context("`STATIC_ASSET_DIR` env var not populated")?;
    let static_asset_dir = PathBuf::from(static_asset_dir);

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
    let stylesheet_content = Arc::<str>::from(stylesheet_content);

    Ok(AppState {
      static_asset_dir,
      stylesheet: stylesheet_content,
    })
  }
}
