use miette::IntoDiagnostic;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{
  EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt,
};

pub fn setup_tracing() -> miette::Result<()> {
  let env_filter = EnvFilter::builder()
    .with_default_directive(LevelFilter::INFO.into())
    .from_env_lossy();
  let formatter = fmt::layer();

  tracing_subscriber::registry()
    .with(formatter)
    .with(env_filter.clone())
    .try_init()
    .into_diagnostic()?;

  tracing::info!(env_filter = env_filter.to_string(), "tracing setup");

  Ok(())
}
