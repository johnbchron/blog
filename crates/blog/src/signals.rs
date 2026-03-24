pub(crate) async fn shutdown_signal() {
  use tokio::signal::{
    ctrl_c,
    unix::{SignalKind, signal},
  };
  let ctrl_c = async {
    ctrl_c().await.expect("failed to install Ctrl+C handler");
  };

  let terminate = async {
    signal(SignalKind::terminate())
      .expect("failed to install signal handler")
      .recv()
      .await;
  };

  tokio::select! {
      _ = ctrl_c => {
        tracing::info!("received SIGINT, shutting down");
      },
      _ = terminate => {
        tracing::info!("received SIGTERM, shutting down");
      },
  }
}
