---
title: "A New Rust Job Runner"
written_on: "2024.09.07"
public: false
---

# Premise
I'm building a service that needs to be able to run jobs. There's going to be some heavy-weight work going on behind the scenes as my service transforms archives, copies between storage locations, chunks and transcodes, etc., so I need a solid solution for building, managing, and running jobs. I haven't built a service like this before. This is the problem at hand.

If I'm going to have a formalized job pipeline for the intense work, it's worth building all the rest of the platform on the same architecture for visibility, persistence, and resilience's sake. Also, committing to building all the non-trivial backend logic on this job system necessitates some nice to have features like launching jobs from other jobs, incremental status reporting, and return values.

I'd expect a high-quality Rust-based solution to this kind of problem to exist, right? I know I'm really constricting my options by demanding that it be in Rust, but what can I say... it's a character flaw at this point.

Oh and one other thing; I don't want a central job-management server. I want the scalability of my runners to only depend on the parallelism of my storage/db, not any other factor. I also just don't want to run another service and increase micro-service dependencies -- I've been pretending to know how to use Kubernetes this whole time and don't want to break my streak.

## Features

So let's set out a list of features that we're wanting.

> I'm saying "let us" and "we" now, implying that I'm dragging you along for this journey. Welcome aboard.

### Must-haves

- Jobs are persistent (not just in memory on the instance where they were spawned)
- Scales horizontally
- Only depends on DB, not on a central job server or cluser
- Generic storage adapter (can use multiple types of DB)
- Job client can await a job in application code (think in an API route)
- Jobs can be cancelled
- Jobs can be automatically retried, and declared dead on too many retries
- Jobs can have return values
- Jobs get IDs that can be returned in user API calls
- Jobs can be recovered from dead runners


### Nice-to-haves

- Jobs can have errors unique to job type
- Jobs can report status unique to job type
- Jobs can easily spawn and await other jobs
- Logs can be aggregated and stored per job invocation

So let's examine the options, shall we? (feel free to skip this section if you don't like bullet points; spoiler alert -- I don't pick one of these)

## The Options

- [`woddle`](https://github.com/zupzup/woddle) - "An async, synchronized database-backed Rust job scheduler"
  - Unclear if it scales horizontally
  - Storage isn't generic (only Postgres)
  - Can't await jobs in application code
  - Jobs can't be cancelled
  - Jobs can't have return values
  - Jobs don't get IDs
  - **Job status can't be checked**
  - Unclear if jobs can be recovered from dead runners
- [`apalis`](https://github.com/geofmureithi/apalis) - "A simple, extensible multithreaded background job processing library for rust"
  - Scales horizontally but runners have to be launched **per job type**
  - **Job status can't be checked**
  - Jobs can't be cancelled
  - Jobs can't have return values
  - Unclear if jobs can be recovered from dead runners
- [`relay`](https://github.com/relay-io/relay) - "A simple no-nonsense job runner"
  - Too ephemeral (doesn't keep job history or logs)
  - No notion of job sucess or fail
  - Jobs can't be recovered from dead runners
  - Jobs can't be automatically retried
- [`gaffer`](https://github.com/survemobility/gaffer) - "Prioritised, parallel job scheduler with concurrent exclusion, job merging, recurring jobs and load limiting for lower priorities"
  - Not persistent
- [`aj`](https://github.com/ikigai-hq/aj) - "Rust - background jobs"
  - No documentation
  - Unclear if it scales horizontally
  - Unclear if jobs can be awaited in application code
  - Unclear if jobs get IDs or have return types
  - Unclear if jobs can be recovered from dead runners

There are others that I could've looked at but my tolerance for missing features and immaturity is obviously significantly higher for first party applications. These criteria are not supposed to be general criteria for persistent, distributed job runners, they're *my* criteria. If I misjudged one of these or there was one that should've made it on the list, please [correct me](mailto:contact@jlewis.sh).

## So Now What?

> "Fine, I'll do it myself." -Thanos

Obviously we're going to build it together, you and I. Or rather, I'll build it in front of you. You know what I mean.

Alright so what are the parts involved? Obviously some kind of `Job` trait, where the implementer describes job parameters and some config, an async method that accepts those parameters for running the job, and a couple associated types for unique job status, return value, and error types. The job "run" function should also take some sort of context reference that lets it interact with the runner and the store. That covers the definition of jobs.

For manipulating jobs, we need some sort of `Client` type that lets you submit, update, read the status of, and cancel jobs. We need a job `Runner` which queries the store and runs jobs that can be run. The runner is also responsible for supplying the context to the job.

The runner should "claim" a job so that we can have multiple runners at the same time, and the runner should also actively tick the job in the job store periodically while the job run function is running so that it can tell when another runner has forfeited a job (the job is "claimed" but was last ticked too long ago). The final responsibility of the runner is to aggregate logs and put them in long-term storage.

Let's get started!

## The `Job` Trait

```sh
cargo new jobs --lib
```

```rust
pub trait Job {

}
```

This is boring. What did I say we need again? Oh right; let's start with the (associated) return type and error type.

```rust
pub trait Job {
  type Result;
  type Error;
}
```

We obviously need to enforce some bounds on these types that allow us to 1) move the job around from thread to thread, and 2) pack the job away into the database and pull it back out again. So that translates to `Send + Sync + 'static` and `Deserialize + Serialize`, respectively.

```rust
use serde::{Deserialize, Serialize};

pub trait Job: for<'a> Deserialize<'a> + Serialize + Send + Sync + 'static {
  type Result: for<'a> Deserialize<'a> + Serialize + Send + Sync + 'static;
  type Error: for<'a> Deserialize<'a> + Serialize + Send + Sync + 'static;
}
```

The `for<'a> Deserialize<'a>` is because `Deserialize` the trait operates on a reference to the type instead of consuming it. We could use `DeserializeOwned` to consume it instead and get rid of the lifetime, but in this case there's no reason to.

These trait bounds are quite verbose though, so let's use a marker trait to be more concise.

```rust
use serde::{Deserialize, Serialize};

pub trait Portable:
  for<'a> Deserialize<'a> + Serialize + Send + Sync + 'static
{
}

impl<T: for<'a> Deserialize<'a> + Serialize + Send + Sync + 'static> Portable
  for T
{
}

pub trait Job: Portable {
  type Result: Portable;
  type Error: Portable;
}
```

So what we're doing here is making a trait `Portable` with all the bounds described earlier, so anything that implements `Portable` is guaranteed to also implement `for<'a> Deserialize<'a> + Ser...`, and the compiler knows this. Then in the second block we're implementing `Portable` for everything that already meets the bounds. The `Portable` trait is now effectively an alias for `for<'a> Deserialize<'a> + Serialize + Send + Sync + 'static`.

Let's add defaults for the associated types, so that we can leave them out when implementing `Job` if we don't need them. Let's set them to the unit type, `()`.

```rust
use serde::{Deserialize, Serialize};

pub trait Portable:
  for<'a> Deserialize<'a> + Serialize + Send + Sync + 'static
{
}

impl<T: for<'a> Deserialize<'a> + Serialize + Send + Sync + 'static> Portable
  for T
{
}

pub trait Job: Portable {
  type Result: Portable = ();
  type Error: Portable = ();
}
```

The compiler tells us:
```sh
error[E0658]: associated type defaults are unstable
  --> crates/jobs/src/lib.rs:22:3
   |
22 |   type Result: Portable = ();
   |   ^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: see issue #29661 <https://github.com/rust-lang/rust/issues/29661> for more information
   = help: add `#![feature(associated_type_defaults)]` to the crate attributes to enable
   = note: this compiler was built on 2024-07-01; consider upgrading it if it is out of date

error[E0658]: associated type defaults are unstable
  --> crates/jobs/src/lib.rs:23:3
   |
23 |   type Error: Portable = ();
   |   ^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: see issue #29661 <https://github.com/rust-lang/rust/issues/29661> for more information
   = help: add `#![feature(associated_type_defaults)]` to the crate attributes to enable
   = note: this compiler was built on 2024-07-01; consider upgrading it if it is out of date

For more information about this error, try `rustc --explain E0658`.
error: could not compile `jobs` (lib) due to 2 previous errors
```

How clear. I guess I don't know why I assumed you could do that; this is even already on `nightly`. The "default" idea comes from generics I think? Anyways, let's check out [that issue](https://github.com/rust-lang/rust/issues/29661).

Reading that tracking issue, it seems that the most general case works (`trait Foo { type Bar = (); }`) but there's some weirdness surrounding defaults for constants defined in traits that use their associated types, and in `dyn` trait objects? Sounds good enough for me. It's just a little ergonomics thing anyways so no biggie to roll it back if it causes an ICE.

```rust
#![feature(associated_type_defaults)]

...
```

We need a method to provide some configuration options, and I'd like it to be a little flexible as to how the user provides it.

```rust
#![feature(associated_type_defaults)]

use serde::{Deserialize, Serialize};

pub struct JobConfig {}

pub trait Portable:
  for<'a> Deserialize<'a> + Serialize + Send + Sync + 'static
{
}

impl<T: for<'a> Deserialize<'a> + Serialize + Send + Sync + 'static> Portable
  for T
{
}

pub trait Job: Portable {
  type Result: Portable = ();
  type Error: Portable = ();

  fn config(&self) -> JobConfig;
}
```

Seems simple enough. Let's add the `run()` function. We're also adding a little `JobContext` struct that we'll pass as a reference into the `run()` function.

For now we'll pass an immutable reference to the `JobContext` and we'll push responsibility onto future us for doing the interior mutability to make that happen. Probably a mutex or something; we won't need a lot of bandwidth out of the context. Whatever -- later.

```rust
#![feature(associated_type_defaults)]

use serde::{Deserialize, Serialize};

pub struct JobConfig {}

pub struct JobContext {}

pub trait Portable:
  for<'a> Deserialize<'a> + Serialize + Send + Sync + 'static
{
}

impl<T: for<'a> Deserialize<'a> + Serialize + Send + Sync + 'static> Portable
  for T
{
}

pub trait Job: Portable {
  type Result: Portable = ();
  type Error: Portable = ();

  fn config(&self) -> &JobConfig;

  async fn run(
    params: Self,
    context: &JobContext,
  ) -> Result<Self::Result, Self::Error>;
}
```

When we use a bare `async fn ...` in a trait, we get a `cargo` warning.

```sh
warning: use of `async fn` in public traits is discouraged as auto trait bounds cannot be specified
  --> crates/jobs/src/lib.rs:27:3
   |
27 |   async fn run(
   |   ^^^^^
   |
   = note: you can suppress this lint if you plan to use the trait only in your own code, or do not care about auto traits l
ike `Send` on the `Future`
   = note: `#[warn(async_fn_in_trait)]` on by default
help: you can alternatively desugar to a normal `fn` that returns `impl Future` and add any desired bounds such as `Send`, but these cannot be relaxed without a breaking API change
   |
27 ~   fn run(
28 |     params: Self,
29 |     context: &JobContext,
30 ~   ) -> impl std::future::Future<Output = Result<Self::Result, Self::Error>> + Send;
   |
```

Once again, how helpful. "Auto trait bounds cannot be specified" could be a little unclear though.

Async functions are just functions that return `impl Future<Output = Something>`. This means that until `Job` is implemented, the return type implementing `Future` is not concrete. It's only specified by trait bounds. So essentially, we only know as much about that return type as we require with trait bounds. Why would we want to know more about that type? Work-stealing executors (like `tokio`) shuffle futures between threads, so they need futures to be `Send + 'static`.

So that lint is saying "you can't make sure that the future returned here is `Send` if you use the `async` syntax sugar, but you probably want to". Which we do, so we'll expand the syntax sugar and add the `Send` bound.

```rust
#![feature(associated_type_defaults)]

use std::future::Future;

use serde::{Deserialize, Serialize};

pub struct JobConfig {}

pub struct JobContext {}

pub trait Portable:
  for<'a> Deserialize<'a> + Serialize + Send + Sync + 'static
{
}

impl<T: for<'a> Deserialize<'a> + Serialize + Send + Sync + 'static> Portable
  for T
{
}

pub trait Job: Portable {
  type Result: Portable = ();
  type Error: Portable = ();

  fn config(&self) -> JobConfig;

  fn run(
    params: Self,
    context: &JobContext,
  ) -> impl Future<Output = Result<Self::Result, Self::Error>> + Send;
}
```

I won't say it's pretty, but it's something. We still need a type for job-specific status.

```rust
pub trait Job: Portable {
  type Status: Portable + Clone = ();
  type Result: Portable = ();
  type Error: Portable = ();

  fn config(&self) -> JobConfig;

  fn run(
    params: Self,
    context: &JobContext,
  ) -> impl Future<Output = Result<Self::Result, Self::Error>> + Send;
}
```

For everything except the `Status` type, I can generally assume we'll only have one copy alive at a given time after deserializing, but we might want to clone and pass around the `Status` type, so I'm tentatively also adding the `Clone` bound.

## The `JobRunner`

The `JobRunner` needs to generic on a storage backend, and it also needs to be able to manage and articulate running jobs.
