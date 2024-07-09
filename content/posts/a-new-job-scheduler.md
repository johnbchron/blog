---
title: "A New Rust Job Scheduler"
written_on: "09/07/24"
public: true
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
