use std::{
  net::SocketAddr,
  task::{Context, Poll},
  time::Instant,
};

use axum::{
  body::Body, extract::ConnectInfo, http::Request, response::Response,
};
use chrono::{DateTime, Utc};
use futures::future::BoxFuture;
use miette::{Context as MietteContext, IntoDiagnostic};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use tokio::sync::mpsc;
use tower::{Layer, Service};

const INIT_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS page_views (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    ts              TEXT    NOT NULL,
    method          TEXT    NOT NULL,
    path            TEXT    NOT NULL,
    query           TEXT,
    status          INTEGER NOT NULL,
    latency_ms      INTEGER NOT NULL,
    response_size   INTEGER,
    user_agent      TEXT,
    referer         TEXT,
    remote_addr     TEXT,
    country         TEXT,
    accept_language TEXT,
    content_type    TEXT,
    is_bot          INTEGER NOT NULL DEFAULT 0,
    ray_id          TEXT
);

CREATE INDEX IF NOT EXISTS idx_pv_ts      ON page_views(ts);
CREATE INDEX IF NOT EXISTS idx_pv_path    ON page_views(path);
CREATE INDEX IF NOT EXISTS idx_pv_country ON page_views(country);
CREATE INDEX IF NOT EXISTS idx_pv_bot     ON page_views(is_bot);
"#;

struct PageView {
  ts:              DateTime<Utc>,
  method:          String,
  path:            String,
  query:           Option<String>,
  status:          u16,
  latency_ms:      u64,
  response_size:   Option<u64>,
  user_agent:      Option<String>,
  referer:         Option<String>,
  remote_addr:     Option<String>,
  country:         Option<String>,
  accept_language: Option<String>,
  content_type:    Option<String>,
  is_bot:          bool,
  ray_id:          Option<String>,
}

#[derive(Debug, Clone)]
pub struct AnalyticsLayer {
  tx: mpsc::Sender<PageView>,
}

impl AnalyticsLayer {
  /// Create the layer and spawn the background writer.
  ///
  /// Works behind Cloudflare, any other reverse proxy, or direct
  /// connections. IP resolution order:
  ///
  /// 1. `CF-Connecting-IP`  (Cloudflare)
  /// 2. `X-Real-IP`         (nginx default)
  /// 3. `X-Forwarded-For`   (first address in chain)
  /// 4. Socket peer addr    (direct connection)
  ///
  /// For the socket fallback, use
  /// `into_make_service_with_connect_info::<SocketAddr>()`.
  pub async fn build(db_url: &str, buffer_size: usize) -> miette::Result<Self> {
    let pool = SqlitePoolOptions::new()
      .max_connections(2)
      .connect(db_url)
      .await
      .into_diagnostic()
      .context("failed to connect to analytics db file")?;

    sqlx::raw_sql(INIT_SQL)
      .execute(&pool)
      .await
      .into_diagnostic()
      .context("failed to prepare analytics db with init SQL")?;

    let (tx, rx) = mpsc::channel::<PageView>(buffer_size);
    tokio::spawn(writer_loop(pool, rx));

    Ok(Self { tx })
  }
}

impl<S> Layer<S> for AnalyticsLayer {
  type Service = AnalyticsService<S>;

  fn layer(&self, inner: S) -> Self::Service {
    AnalyticsService {
      inner,
      tx: self.tx.clone(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct AnalyticsService<S> {
  inner: S,
  tx:    mpsc::Sender<PageView>,
}

impl<S> Service<Request<Body>> for AnalyticsService<S>
where
  S: Service<Request<Body>, Response = Response> + Clone + Send + 'static,
  S::Future: Send + 'static,
{
  type Error = S::Error;
  type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;
  type Response = Response;

  fn poll_ready(
    &mut self,
    cx: &mut Context<'_>,
  ) -> Poll<Result<(), Self::Error>> {
    self.inner.poll_ready(cx)
  }

  fn call(&mut self, req: Request<Body>) -> Self::Future {
    let headers = req.headers();

    let method = req.method().to_string();
    let path = req.uri().path().to_owned();
    let query = req.uri().query().map(String::from);

    let user_agent = header_str(headers, "user-agent");
    let referer = header_str(headers, "referer");
    let accept_language = header_str(headers, "accept-language");

    // cloudflare headers
    let country = header_str(headers, "cf-ipcountry");
    let ray_id = header_str(headers, "cf-ray");

    let remote_addr = resolve_ip(headers, req.extensions());

    let is_bot = user_agent
      .as_deref()
      .map(woothee::is_crawler)
      .unwrap_or(true); // no UA → assume bot

    let tx = self.tx.clone();

    // we have clone the service and call the clone because poll_ready was
    // called on self, not the clone.
    // https://docs.rs/tower/latest/tower/trait.Service.html#be-careful-when-cloning-inner-services
    let clone = self.inner.clone();
    let mut inner = std::mem::replace(&mut self.inner, clone);

    Box::pin(async move {
      let start = Instant::now();
      let resp = inner.call(req).await?;
      let latency_ms = start.elapsed().as_millis() as u64;

      let response_size = resp
        .headers()
        .get(axum::http::header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok());

      let content_type = resp
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(String::from);

      let _ = tx.try_send(PageView {
        ts: Utc::now(),
        method,
        path,
        query,
        status: resp.status().as_u16(),
        latency_ms,
        response_size,
        user_agent,
        referer,
        remote_addr,
        country,
        accept_language,
        content_type,
        is_bot,
        ray_id,
      });

      Ok(resp)
    })
  }
}

fn resolve_ip(
  headers: &axum::http::HeaderMap,
  extensions: &axum::http::Extensions,
) -> Option<String> {
  // cloudflare
  if let Some(ip) = header_str(headers, "cf-connecting-ip") {
    return Some(ip);
  }

  // nginx / generic proxy
  if let Some(ip) = header_str(headers, "x-real-ip") {
    return Some(ip);
  }

  // X-Forwarded-For
  if let Some(chain) = header_str(headers, "x-forwarded-for")
    && let Some(first) = chain.split(',').next()
  {
    let trimmed = first.trim();
    if !trimmed.is_empty() {
      return Some(trimmed.to_owned());
    }
  }

  // socket peer addr
  extensions
    .get::<ConnectInfo<SocketAddr>>()
    .map(|ci| ci.0.ip().to_string())
}

async fn writer_loop(pool: SqlitePool, mut rx: mpsc::Receiver<PageView>) {
  let mut batch: Vec<PageView> = Vec::with_capacity(64);

  loop {
    let deadline = tokio::time::sleep(std::time::Duration::from_millis(500));
    tokio::pin!(deadline);

    loop {
      tokio::select! {
          Some(ev) = rx.recv() => {
              batch.push(ev);
              if batch.len() >= 64 { break; }
          }
          _ = &mut deadline => break,
          else => return,
      }
    }

    if batch.is_empty() {
      continue;
    }

    if let Err(e) = flush(&pool, &batch).await {
      tracing::error!(count = batch.len(), err = %e, "analytics flush failed");
    }

    batch.clear();
  }
}

async fn flush(
  pool: &SqlitePool,
  batch: &[PageView],
) -> Result<(), sqlx::Error> {
  let mut tx = pool.begin().await?;

  for ev in batch {
    sqlx::query(
      r#"INSERT INTO page_views
                   (ts, method, path, query, status, latency_ms, response_size,
                    user_agent, referer, remote_addr, country,
                    accept_language, content_type, is_bot, ray_id)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
    )
    .bind(ev.ts.to_rfc3339())
    .bind(&ev.method)
    .bind(&ev.path)
    .bind(&ev.query)
    .bind(ev.status as i64)
    .bind(ev.latency_ms as i64)
    .bind(ev.response_size.map(|s| s as i64))
    .bind(&ev.user_agent)
    .bind(&ev.referer)
    .bind(&ev.remote_addr)
    .bind(&ev.country)
    .bind(&ev.accept_language)
    .bind(&ev.content_type)
    .bind(ev.is_bot)
    .bind(&ev.ray_id)
    .execute(&mut *tx)
    .await?;
  }

  tx.commit().await
}

fn header_str(headers: &axum::http::HeaderMap, name: &str) -> Option<String> {
  headers
    .get(name)
    .and_then(|v| v.to_str().ok())
    .map(String::from)
}
