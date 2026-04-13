+++
title = "absurd-rs - an absurd plan"
date = "2026-04-13"
+++

## Overview

Rust SDK for [earendil-works/absurd](https://github.com/earendil-works/absurd): a Postgres-native
durable workflow system. Wraps the `absurd.*` stored-procedure surface with async task
registration, worker polling, checkpointed steps, event suspension, and sleep.

---

## Dependencies

| Crate | Features |
|---|---|
| `tokio` | `rt`, `time`, `sync`, `macros` |
| `tokio-util` | `sync` |
| `sqlx` | `postgres`, `runtime-tokio-rustls`, `uuid`, `chrono`, `json` |
| `serde` + `serde_json` | |
| `uuid` | `v4` |
| `chrono` | `serde` |
| `thiserror` | |
| `tracing` | |

---

## Public API

### Error type

```rust
#[derive(Debug, thiserror::Error)]
pub enum AbsurdError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("task not found: {0}")]
    TaskNotFound(String),
    #[error("timeout waiting for {0}")]
    Timeout(String),
    #[error("queue name invalid: {0}")]
    InvalidQueueName(String),
    #[error("no task context")]
    NoTaskContext,
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
    // Internal control-flow — not exposed through pub API
    Suspend,
    Cancelled,
    FailedRun,
}
```

### Client

```rust
pub struct ClientBuilder { /* opaque */ }

impl ClientBuilder {
    // Reads ABSURD_DATABASE_URL / PGDATABASE as fallback.
    pub fn database_url(self, url: impl Into<String>) -> Self;
    pub fn queue_name(self, name: impl Into<String>) -> Self;
    pub fn default_max_attempts(self, n: u32) -> Self;
    pub fn max_connections(self, n: u32) -> Self;
    pub async fn build(self) -> Result<Client, AbsurdError>;
}

pub struct Client { /* opaque */ }

impl Client {
    pub fn builder() -> ClientBuilder;
    pub fn from_pool(pool: PgPool, queue_name: impl AsRef<str>) -> Result<Self, AbsurdError>;
    pub fn queue_name(&self) -> &str;

    pub async fn create_queue(&self, queue_name: impl AsRef<str>) -> Result<(), AbsurdError>;
    pub async fn drop_queue(&self, queue_name: impl AsRef<str>) -> Result<(), AbsurdError>;
    pub async fn list_queues(&self) -> Result<Vec<String>, AbsurdError>;

    pub fn register_task(&self, handler: impl TaskHandler) -> Result<(), AbsurdError>;

    pub async fn spawn<P: Serialize>(
        &self,
        task_name: impl AsRef<str>,
        params: &P,
        options: SpawnOptions,
    ) -> Result<SpawnResult, AbsurdError>;

    pub async fn emit_event<P: Serialize>(
        &self,
        queue_name: impl AsRef<str>,
        event_name: impl AsRef<str>,
        payload: &P,
    ) -> Result<(), AbsurdError>;

    pub async fn fetch_task_result(
        &self,
        queue_name: impl AsRef<str>,
        task_id: Uuid,
    ) -> Result<Option<TaskResultSnapshot>, AbsurdError>;

    /// Pass `Duration::MAX` for no timeout.
    pub async fn await_task_result(
        &self,
        queue_name: impl AsRef<str>,
        task_id: Uuid,
        timeout: Duration,
    ) -> Result<TaskResultSnapshot, AbsurdError>;

    pub async fn retry_task(
        &self,
        queue_name: impl AsRef<str>,
        task_id: Uuid,
        options: RetryTaskOptions,
    ) -> Result<SpawnResult, AbsurdError>;

    pub async fn cancel_task(
        &self,
        queue_name: impl AsRef<str>,
        task_id: Uuid,
    ) -> Result<(), AbsurdError>;

    pub async fn cleanup_tasks(
        &self,
        queue_name: impl AsRef<str>,
        ttl: Duration,
        limit: u32,
    ) -> Result<u32, AbsurdError>;

    pub async fn cleanup_events(
        &self,
        queue_name: impl AsRef<str>,
        ttl: Duration,
        limit: u32,
    ) -> Result<u32, AbsurdError>;

    pub fn worker(&self) -> WorkerBuilder;
}
```

### Task registration

```rust
pub trait TaskHandler: Send + Sync + 'static {
    type Params: DeserializeOwned + Send;
    type Result: Serialize + Send;

    fn name(&self) -> &str;
    fn options(&self) -> TaskOptions { TaskOptions::default() }

    fn run(
        &self,
        params: Self::Params,
        ctx: TaskContext,
    ) -> impl Future<Output = Result<Self::Result, AbsurdError>> + Send;
}

#[derive(Default)]
pub struct TaskOptions {
    pub queue_name: Option<String>,
    pub default_max_attempts: Option<u32>,
    pub default_cancellation: Option<CancellationPolicy>,
}

/// Closure-based shortcut; for simple cases that don't need a named struct.
pub fn task<P, R, F, Fut>(
    name: impl Into<String>,
    options: TaskOptions,
    f: F,
) -> impl TaskHandler
where
    P: DeserializeOwned + Send + 'static,
    R: Serialize + Send + 'static,
    F: Fn(P, TaskContext) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<R, AbsurdError>> + Send + 'static;
```

### Spawn options and result

```rust
#[non_exhaustive]
pub struct SpawnOptions { /* private fields */ }

impl SpawnOptions {
    pub fn builder() -> SpawnOptionsBuilder;
}

pub struct SpawnOptionsBuilder { /* opaque */ }

impl SpawnOptionsBuilder {
    pub fn queue_name(self, q: impl Into<String>) -> Self;
    pub fn max_attempts(self, n: u32) -> Self;
    pub fn retry_strategy(self, s: RetryStrategy) -> Self;
    pub fn headers(self, h: serde_json::Value) -> Self;
    pub fn cancellation(self, p: CancellationPolicy) -> Self;
    pub fn idempotency_key(self, k: impl Into<String>) -> Self;
    pub fn build(self) -> SpawnOptions;
}

#[non_exhaustive]
pub struct SpawnResult {
    pub task_id: Uuid,
    pub run_id: Uuid,
    pub attempt: u32,
    pub created: bool,
}
```

Registered queue and defaults are picked up automatically; unregistered tasks require
`SpawnOptionsBuilder::queue_name`.

### Task result

```rust
pub enum TaskState { Pending, Running, Sleeping, Completed, Failed, Cancelled }

#[non_exhaustive]
pub struct TaskResultSnapshot {
    pub state: TaskState,
    pub result: Option<serde_json::Value>,
    pub failure: Option<serde_json::Value>,
}

impl TaskResultSnapshot {
    pub fn is_terminal(&self) -> bool;
    pub fn decode_result<T: DeserializeOwned>(&self) -> Result<Option<T>, AbsurdError>;
    pub fn decode_failure<T: DeserializeOwned>(&self) -> Result<Option<T>, AbsurdError>;
}
```

Polling uses exponential backoff (50 ms → 1 s).

### TaskContext

```rust
pub struct TaskContext { /* opaque */ }

impl TaskContext {
    pub fn task_id(&self) -> Uuid;
    pub fn run_id(&self) -> Uuid;
    pub fn task_name(&self) -> &str;
    pub fn queue_name(&self) -> &str;
    pub fn attempt(&self) -> u32;
    pub fn headers(&self) -> &serde_json::Value;

    pub async fn step<T, E, F, Fut>(
        &self,
        name: impl AsRef<str>,
        f: F,
    ) -> Result<T, AbsurdError>
    where
        T: Serialize + DeserializeOwned + Send,
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>> + Send;

    /// Use with `complete_step` to checkpoint before intentionally failing.
    pub async fn begin_step<T: DeserializeOwned + Send>(
        &self,
        name: impl AsRef<str>,
    ) -> Result<StepHandle<T>, AbsurdError>;

    pub async fn complete_step<T: Serialize + DeserializeOwned + Send>(
        &self,
        handle: StepHandle<T>,
        value: T,
    ) -> Result<T, AbsurdError>;

    pub async fn sleep_for(
        &self,
        step_name: impl AsRef<str>,
        d: Duration,
    ) -> Result<(), AbsurdError>;

    pub async fn sleep_until(
        &self,
        step_name: impl AsRef<str>,
        wake_at: DateTime<Utc>,
    ) -> Result<(), AbsurdError>;

    /// Pass `Duration::MAX` for no timeout.
    pub async fn await_event<T: DeserializeOwned + Send>(
        &self,
        event_name: impl AsRef<str>,
        timeout: Duration,
    ) -> Result<T, AbsurdError>;

    /// Must target a different queue to avoid worker-slot deadlock.
    /// Pass `Duration::MAX` for no timeout.
    pub async fn await_task_result(
        &self,
        queue_name: impl AsRef<str>,
        task_id: Uuid,
        timeout: Duration,
    ) -> Result<TaskResultSnapshot, AbsurdError>;

    /// Pass `None` to reuse the original claim timeout.
    pub async fn heartbeat(&self, extension: Option<Duration>) -> Result<(), AbsurdError>;

    pub async fn emit_event<P: Serialize>(
        &self,
        event_name: impl AsRef<str>,
        payload: &P,
    ) -> Result<(), AbsurdError>;
}

pub enum StepHandle<T> {
    Done { name: String, checkpoint_name: String, state: T },
    Pending { name: String, checkpoint_name: String },
}

impl<T> StepHandle<T> {
    pub fn is_done(&self) -> bool;
    pub fn state(&self) -> Option<&T>;
}
```

### Worker

```rust
pub struct WorkerBuilder { /* opaque */ }

impl WorkerBuilder {
    pub fn worker_id(self, id: impl Into<String>) -> Self;
    pub fn claim_timeout(self, d: Duration) -> Self;      // default: 120 s
    pub fn batch_size(self, n: usize) -> Self;            // default: concurrency
    pub fn concurrency(self, n: usize) -> Self;           // default: 1
    pub fn poll_interval(self, d: Duration) -> Self;      // default: 250 ms
    pub fn fatal_on_lease_timeout(self, b: bool) -> Self; // default: true
    pub fn start(self) -> WorkerHandle;
}

pub struct WorkerHandle { /* opaque */ }

impl WorkerHandle {
    pub async fn shutdown(self);
    /// Claim and execute one batch; useful for testing.
    pub async fn work_batch(&self) -> Result<(), AbsurdError>;
}

impl Drop for WorkerHandle {
    fn drop(&mut self) { /* cancel the internal CancellationToken */ }
}
```

Concurrency via `tokio::sync::Semaphore`; shutdown via `CancellationToken` + `JoinSet`. Task
execution errors are logged with `tracing::error!`; lease expiry emits `tracing::warn!` at 1×
claim timeout and calls `process::exit(1)` at 2× when `fatal_on_lease_timeout` is set.

---

## Design Notes

- **Builders everywhere** — `Client`, `SpawnOptions`, `WorkerBuilder` use the builder pattern;
  no public config structs with public fields.
- **`impl AsRef<str>`** on all queue/task/event name parameters.
- **`Duration::MAX`** as the "no timeout" sentinel instead of `Option<Duration>`.
- **`TaskHandler` trait** for named, reusable, statically-typed task definitions; `task()` free
  function as a closure shortcut.
- **`StepHandle<T>` as an enum** (`Done`/`Pending`) — exhaustive matching prevents access to
  `state` when the step hasn't been completed.
- **`begin_step`/`complete_step`** for the pattern of checkpointing before intentionally failing,
  so the step is skipped on retry.
- **`tracing` only** — no `on_error` callback; `wrap_task_execution` hook handles custom
  observability.
- **`WorkerHandle`** is returned immediately from `start()`; the polling loop runs on a spawned
  task. `shutdown().await` joins in-flight work; `Drop` cancels without waiting.
- **Hooks** — `before_spawn` and `wrap_task_execution` for correlation-ID propagation and tracing.

---

## File Structure

```
crates/absurd/
  src/
    lib.rs       # re-exports
    client.rs    # Client, ClientBuilder
    worker.rs    # WorkerBuilder, WorkerHandle
    context.rs   # TaskContext, StepHandle
    handler.rs   # TaskHandler trait, task(), type-erased registry
    types.rs     # SpawnOptions, SpawnResult, TaskResultSnapshot, TaskState, etc.
    error.rs     # AbsurdError
    db.rs        # sqlx query helpers (private)
    util.rs      # validate_queue_name, backoff
  Cargo.toml
```

---

## Implementation Phases

1. **Scaffold** — `Cargo.toml`, `error.rs`, `types.rs`, `util.rs`
2. **DB layer** — `db.rs`: typed sqlx wrappers for all `absurd.*` stored procedures
3. **Client** — `client.rs`: builder, queue ops, spawn, events, results, retry/cancel/cleanup
4. **Task handler** — `handler.rs`: `TaskHandler` trait, `task()`, type-erased registry
5. **TaskContext** — `context.rs`: steps, `begin_step`/`complete_step`, sleep, events, heartbeat
6. **Worker** — `worker.rs`: claim loop, semaphore, lease watchdog, graceful shutdown
7. **Integration tests** — gated behind a feature flag; requires Postgres with absurd schema
8. **Examples** — `examples/order_fulfillment.rs`
