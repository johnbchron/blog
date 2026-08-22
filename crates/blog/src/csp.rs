//! Content Security Policy.
//!
//! Every response gets a strict CSP header. The three inline blocks we emit
//! (the columbo swap script and the two inline `<style>` blocks in
//! [`crate::page_wrapper`]) are authorized with a per-request nonce rather
//! than `'unsafe-inline'`, so injected inline scripts / event handlers (e.g.
//! from markdown rendered by the playground) are still blocked.

use axum::{
  body::Body,
  http::{HeaderValue, Request, header},
  middleware::Next,
  response::Response,
};
use nanorand::{Rng, WyRand};

/// The per-request CSP nonce, shared between the response header and the
/// inline tags via request extensions.
#[derive(Clone)]
pub struct CspNonce(pub String);

/// Nonce length in characters. 24 alphanumeric chars is ~142 bits of entropy,
/// comfortably beyond guessing.
const NONCE_LEN: usize = 24;
const NONCE_ALPHABET: &[u8] =
  b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

fn generate_nonce() -> String {
  let mut rng = WyRand::new();
  (0..NONCE_LEN)
    .map(|_| NONCE_ALPHABET[rng.generate_range(0..NONCE_ALPHABET.len())] as char)
    .collect()
}

/// Builds the policy string for a given nonce.
///
/// `default-src 'none'` makes `object-src`, `frame-src`, `worker-src`, etc.
/// all resolve to none. `base-uri`, `form-action`, and `frame-ancestors` do
/// *not* fall back to `default-src`, so they're listed explicitly.
fn policy(nonce: &str) -> String {
  format!(
    "default-src 'none'; \
     script-src 'self' 'nonce-{nonce}'; \
     style-src 'self' 'nonce-{nonce}'; \
     font-src 'self'; \
     img-src 'self' data:; \
     connect-src 'self'; \
     base-uri 'none'; \
     form-action 'self'; \
     frame-ancestors 'none'"
  )
}

/// Middleware: mint a nonce, expose it to the handler via request extensions,
/// and stamp the CSP header on the response.
pub async fn apply_csp(mut req: Request<Body>, next: Next) -> Response {
  let nonce = generate_nonce();
  req.extensions_mut().insert(CspNonce(nonce.clone()));

  let mut resp = next.run(req).await;

  if let Ok(value) = HeaderValue::from_str(&policy(&nonce)) {
    resp
      .headers_mut()
      .insert(header::CONTENT_SECURITY_POLICY, value);
  }

  resp
}
