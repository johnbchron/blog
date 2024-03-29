---
title: "Strongly-typed IDs in SurrealDB"
written_on: "18/03/24"
public: true
---

I've recently begun to use [SurrealDB][1] more. I first used it mostly for hobby projects to test it out, but now I'm working on at least one production-ish application that uses SurrealDB, which means my requirements have changed somewhat. As I develop, more of my concern is placed on the security and future-proofing side of things, as well as ergonomics for future developers. I want to develop good systems and paradigms for that are extensible and uniform. Doing so requires more thought than I previously devoted to using Surreal.

One such area is in DB models, i.e. what structures our DB data goes into when it enters our application. I write applications using Surreal primarily in Rust, in which it's essential to properly (and explicitly) specify your data structures beforehand. Rust also allows for strict type safety, which we can utilize in our code to make sure that IDs for different models aren't contaminated. This will be the topic for today's post. Let's walk through how to set up strongly-typed IDs using Rust and Surreal.

## Starting out

Let's begin with a model. Let's say for the sake of the argument that we're building an app where you can buy and sell photos, and in that app we have the following semantic items:
- `Artifact` - Anything that goes in object storage.
- `Photo` - Represents a single photo. Has an `original` artifact.
- `PhotoGroup` - A group of photos. This is the unit that can be bought.
- `User` - You know what a user is.

> Typically these sorts of examples start out with lame items like people, dogs, cats, etc. This is still lame but at least it will probably be easier to see how this practice can be implemented in actual applications. I'm doing my best here.

We want to start representing these items in our code. Let's make a struct for our `Artifact` item.

```rust
pub struct Artifact {
  object_store_key: String,
}
```

This is easy-peasy because an `Artifact` doesn't depend on anything within the database. It only depends on some intangible object store, and for the sake of the argument we'll say that everything in the object store is immutable (because we can do whatever we want; it's a hypothetical, remember?).

Let's make the struct for our `Photo` item.

```rust
pub struct Photo {
  original: Artifact
}
```

Hold on one second. I *guess* we can do this, but it seems like a big decision.

### To Table or Not to Table

What we've done here is **inlined** the `Artifact` inside the `Photo`. What we're communicating with this structure is that every `Artifact` is tied to exactly one `Photo`, so no table and no IDs. There are pros and cons to every decision whether or not to build a table for some item, but the decision basically comes down to the context in which you need that information, the ownership of the information, and the lifetime of the information.

In the case of an `Artifact` within a `Photo`, we're going to need it anytime we want to display a `Photo`, so ✓ for context. As for what the ownership and lifetime of a `Artifact` should be, at first it seems like a `Photo` should own and control the lifetime of an `Artifact`. However, an `Artifact` should theoretically be more general than always being owned by a `Photo`, if we're building a whole application. Or maybe it's better to say that we're losing something by restricting every `Artifact` to being owned by a `Photo`, because maybe later we could use `Artifact`s to hold images that aren't part of a `Photo`, like a profile picture. Anyhow, let's give artifacts their own table.

How should we represent that? We'll probably need an ID somewhere. Let's just go ahead and put an ID for the `Artifact` in both places, inside the `Artifact` and the `Photo`. For the `Photo`, it's to know which `Artifact` we *need* to retrieve, and for the `Artifact`, it's to know which `Artifact` we *have* retrieved.

We'll throw in a "self" ID for the `Photo` as well because it'll definitely have its own table.

```rust
pub struct Artifact {
  id: String,
  object_store_key: String,
}

pub struct Photo {
  id: String,
  original_id: String,
}
```

So this is better, but there's definitely work to be done. `String` is insufficient I think.

> "Wait... is `original_id` for the `Artifact` called 'original', or is it for the 'original' photo ID?", you ask with concern in your voice.

Ah, good question. Another thing to address. The root of the problem behind the concern you raised is that neither the name `original_id` nor the type `String` provide any information about the table the ID is referencing. We can do better. Let's start by not addressing the fact that the backing type for our IDs is a `String`, and just deal with "flavoring" our IDs with the table they reference.

## Beginning to Type Our IDs

We'll start by using the [newtype][newtype] pattern to wrap our IDs, with one new type for each table.

```rust
pub struct ArtifactRecordId(String);

impl ArtifactRecordId {
  pub fn new(inner: String) -> ArtifactRecordId { ArtifactRecordId(inner) }
}

pub struct PhotoRecordId(String);

impl PhotoRecordId {
  pub fn new(inner: String) -> PhotoRecordId { PhotoRecordId(inner) }
}

pub struct Artifact {
  id: ArtifactRecordId,
  object_store_key: String,
}

impl Artifact {
  pub fn new(id: ArtifactRecordId, object_store_key: String) -> Artifact {
    Artifact { id, object_store_key }
  }
}

pub struct Photo {
  id: PhotoRecordId,
  original_id: ArtifactRecordId,
}

impl Photo {
  pub fn new(id: PhotoRecordId, original_id: ArtifactRecordId) -> Photo {
    Photo { id, original_id }
  }
}
```

Wow, much better. Firstly, it looks much more rusty[^1]. Generally if you're throwing a bunch of strings around in a Rust item, that might be a sign it's time to strengthen your types.

So what are the benefits here? Firstly, it solves our problem of documentation. When I look at the definition for the `Photo` type, the kind of thing that the `original_id` is refering to is unambiguous: it's an `ArtifactRecordId` so it must be referring to an `Artifact`. Of course we could have just written actual documentation here, but it is good for our code to be clear on its own as well.

Secondly, using a dedicated type here lets us take advantage of the type system, which is one of Rust's powerful defenses against incorrect code. Very rusty.

Say I tried to do the following (i.e. I forget what kind of thing `ambiguous_id` is referring to when I'm writing some function):

```rust
let original_id = ArtifactRecordId::new("TheOneRing".to_owned());
let ambiguous_id = ArtifactRecordId::new("42".to_owned());

let photo = Photo::new(ambiguous_id, original_id);
```

I get the following error:

```
error[E0308]: mismatched types
  --> src/main.rs:33:28
   |
33 |     let photo = Photo::new(ambiguous_id, original_id);
   |                 ---------- ^^^^^^^^^^^^ expected `PhotoRecordId`, found `ArtifactRecordId`
   |                 |
   |                 arguments to this function are incorrect
```

This is very very nice. Now as long as we serialize and deserialize our Rust structs to and from the right tables, we will never mismatch IDs. Which reminds me...

## De/Serializing

These DB models are useless unless we can serialize and deserialize to and from the DB and other places. Up until now we haven't interfaced with Surreal at all. We'll also bring in our first dependency, `serde`. Let's derive some things.

```rust
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ArtifactRecordId(String);

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PhotoRecordId(String);

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Artifact {
  id: ArtifactRecordId,
  object_store_key: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Photo {
  id: PhotoRecordId,
  original_id: ArtifactRecordId,
}
```

Now let's fire up Surreal and use in-memory mode. We'll just create an `Artifact`, put it into surreal, and then take it back out and inspect its contents.

```shell
$ cargo add surrealdb --features kv-mem
$ cargo add serde --features derive
$ cargo add tokio --features full
```

```rust
#[tokio::main]
async fn main() -> surrealdb::Result<()> {
  // make an artifact
  let artifact = Artifact::new(
    ArtifactRecordId::new("iYZD1JS7XDypxU5i".to_string()),
    "jp2Lb6511f3Y2qVm".to_string(),
  );

  // debug the artifact before it goes into surreal
  dbg!(&artifact);

  // create the db
  let db = Surreal::new::<Mem>(()).await?;
  db.use_ns("test").use_db("test").await?;

  // put it into surreal
  let created: Option<Artifact> = db
    // pretend we made this `inner()` method for the IDs
    .create(("artifact", artifact.id.inner().clone()))
    .content(artifact)
    .await?;

  // debug the artifact after it comes out of surreal
  dbg!(created);

  Ok(())
}
```

When we run this, we get

```
[src/main.rs:55:3] &artifact = Artifact {
    id: ArtifactRecordId(
        "iYZD1JS7XDypxU5i",
    ),
    object_store_key: "jp2Lb6511f3Y2qVm",
}
Error: Db(IdMismatch { value: "'iYZD1JS7XDypxU5i'" })
```

Oof! We didn't even get far enough to compare the two, as we failed to create the record.

## SurrealDB IDs

We got an ID mismatch. This is because Surreal is clever with its ser/de. If you have a field named `id`, Surreal will attempt to use it as a Surreal [`Thing`][2]. It will also work if your `id` field is single-value tuple struct wrapping a `Thing`. By the way, a `Thing` looks like this:

```rust
pub struct Thing {
  pub tb: String,
  pub id: Id,
}
```

What's the mysterious `Id` there you ask? Why, it's an enum!

```rust
pub enum Id {
  Number(i64),
  String(String),
  Array(Array),
  Object(Object),
  Generate(Gen),
}
```

We're not going to use any of that directly though so I'll just move on. Let's try remodeling our strongly typed IDs to contain `Thing` instead of `String`.

```rust
pub struct ArtifactRecordId(Thing);
```

```rust
pub struct PhotoRecordId(Thing);
```

```rust
#[tokio::main]
async fn main() -> surrealdb::Result<()> {
  let artifact = Artifact::new(
    ArtifactRecordId::new(Thing {
      tb: "artifact".to_string(),
      id: "iYZD1JS7XDypxU5i".into(),
    }),
    "jp2Lb6511f3Y2qVm".to_string(),
  );

  [cut]
  
  let created: Option<Artifact> = db
    .create(artifact.id.inner()).content(artifact).await?;

  [cut]
}
```

When run, we get

```
[src/main.rs:58:3] &artifact = Artifact {
    id: ArtifactRecordId(
        Thing {
            tb: "artifact",
            id: String(
                "iYZD1JS7XDypxU5i",
            ),
        },
    ),
    object_store_key: "jp2Lb6511f3Y2qVm",
}
[src/main.rs:71:3] created = Some(
    Artifact {
        id: ArtifactRecordId(
            Thing {
                tb: "artifact",
                id: String(
                    "iYZD1JS7XDypxU5i",
                ),
            },
        ),
        object_store_key: "jp2Lb6511f3Y2qVm",
    },
)
```

Awesome. This works, and we can clean up the ergonomics pretty easily. We can use the ID directly in the `.create()` method by implementing a trait, and we can also make the ID `Copy` because `Thing` is `Copy`. Then the create statement looks like this:

```rust
let created: Option<Artifact> = db.create(artifact.id).content(artifact).await?;
```

The trait for using the ID directly is this:

```rust
use surrealdb::{
  Error,
  opt::{IntoResource, Resource},
  sql::{Id, Thing},
};

impl<R> IntoResource<Option<R>> for ArtifactId {
  fn into_resource(self) -> Result<Resource, Error> {
    Ok(Resource::RecordId(Thing {
      tb: "artifact".to_string(),
      id: Id::String(self.0.to_string())
    }))
  }
}
```

That seems ergonomic enough.

## The Next Problem

Now that we've got IDs and models it makes sense to separate them out into their own library, since the whole point of them existing is shared functionality. Maybe you'll have a workspace where you have a `core_types` crate that contains your IDs and models. Perfect. Maybe you'll want to share that with your Rust frontend, since you'll be sending some of these types over the network and you'd love to not repeat yourself.

You can do that, but you'll wake up to a rude surprise. The `surrealdb` library -- you know, the one your `Thing` type comes from -- it weighs a hefty **336 dependencies** at the time of writing. This is absolutely unacceptable for embedding within a Rust-compiled WASM bundle, so we'll look for alternatives.

The `surrealdb` library offers no feature flags that restrict it down to any manageable size, so the only option is to not include it in whatever version of your `core_types` library goes into your frontend package. How will we cut it out but still use the `Thing` type to de/serialize? This is the challenge.

---

For me, this is where a lot of dead-end experimentation happened. One of the things I discovered is that the `id` field cleverness only requires that the type that occupies it can be deserialized from a `Thing`. If it serializes to a string (without a table prefix), that's fine; surreal will parse it into a `Thing`. Remember that this only applies to the `id` field though. If you follow this schema, strongly-typed IDs in non-`id` fields will only exist as strings without the table prefix. This is mostly fine though.

## The Next Solution

I won't walk you through all of it (frankly partially because I don't remember all of what I did), but I'll share my solution with you from the point that I encountered this problem.

Essentially, feature flags are extremely underrated. The approach that I settled on is the following:
 1. Make a "server-side" feature flag for the `core_types` library.
 2. Find a strong backing ID type that is compatible with Surreal's inner `Id` type (like ULID).
 3. Create wrapper ID types around your backing ID for each model you have, and include them in each model as I did above.
 4. Gated by the "server-side" feature flag, do the following:
     1. Add the `surrealdb` dependency
     2. Allow your ID types to deserialize from `Thing` or your backing ID type.

The backing ID is just something to reliably generate primary keys, since requiring them to be in all our models before we send them to Surreal requires us to make them ourselves. [ULID][3] does a good job of that and is simple and light.

The biggest challenge here is to allow deserializing your backing ID type from **either** `Thing` or your backing ID's serialized form, at runtime, with no other information. It's pretty easy though with an intermediary type and a bit of `serde` magic. Here's what that looks like; I'll pick `ulid` for the example.

```rust
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "ssr", serde(from = "ssr::UserRecordIdOrThing"))]
pub struct UserRecordId(ulid::Ulid);

#[cfg(feature = "ssr")]
mod ssr {
  #[derive(Deserialize, PartialEq, Debug, Clone)]
  #[serde(untagged)]
  pub enum UserRecordIdOrThing {
    Thing(Thing),
    UserRecordId(ulid::Ulid),
  }

  impl From<UserRecordIdOrThing> for UserRecordId {
    fn from(uiot: UserRecordIdOrThing) -> Self {
      match uiot {
        UserRecordIdOrThing::UserRecordId(id) => UserRecordId(id),
        UserRecordIdOrThing::Thing(thing) => {
          UserRecordId(ulid::Ulid::from_str(&thing.id.to_string()).unwrap())
        }
      }
    }
  }
}
```

I'll walk you through this.

- On our `UserRecordId`, the `cfg_attr` attribute says "when on feature `ssr`, deserialize the given value as a `ssr::UserRecordIddOrThing`, and then call `UserRecordId::from()` on the result".
- Our `UserRecordId` contains a `Ulid`.
- In our `ssr` module (which is only active when the `ssr` feature is active; `Cargo.toml` not shown):
  - `UserRecordIdOrThing` has the `#[serde(untagged)]` attribute, which controls how `serde` handles enums, and in this case says "just look at the field types and guess which one it is". This has the (desired) side effect of allowing us to deserialize a `UserRecordIdOrThing` from either a `UserRecordId` or `Thing`.
  - We can get a `UserRecordId` from a `UserRecordIdOrUlid`.

When we deserialize a field which came from a `Thing` into a `UserRecordId`, `serde` will attempt to deserialize the value to a `UserRecordIdOrThing`. It will see the `tb` and `id` struct fields within the value, will not look for a tag (because of the `untagged` bit), and will match that combination of fields to `UserRecordIdOrThing::Thing`. If instead it sees only a `String`, it'll match to `UserRecordIdOrThing::UserRecordId` and attempt to parse the `String` to a `Ulid`. Finally, it will convert the `UserRecordIdOrThing` value to a `UserRecordId` value.

So now, when the `ssr` feature is disabled, the `UserRecordId` is a plain wrapper around a plain `Ulid`, and there is no dependency on `surrealdb`. When the `ssr` feature is enabled, that same value and type can be correctly serialized and deserialized to/from Surreal, with the cost of the `surrealdb` dependency. Exactly what we wanted.

## Wrapping Up

If you feel like this is a lot of boilerplate, you're right, but it's worth it. You can switch `UserRecordIdOrThing` to `UlidOrThing` to reduce boilerplate for multiple ID types (I did this in my implementation), but I wrote the blog post the other way and didn't want to rewrite it. You can also reduce implementation boilerplate using a simple `macro_rules!` macro -- I'd love to provide an awesome resource here but I learned `macro_rules!` through experimentation, so let me look for one quickly... ah, [here](https://doc.rust-lang.org/reference/macros-by-example.html) we are.

Anyways, I hope this was useful to you! I am actively building a production application with Surreal and Rust top-to-bottom (as you might have guessed), so I would love any questions or suggestions you might have. You can email me [here](mailto:blog@jlewis.sh). Thanks for reading!

---

[^1]: Python definitely beats us with "pythonic", but I think "rusty" is more endearing.

[1]: https://surrealdb.com/
[newtype]: https://doc.rust-lang.org/rust-by-example/generics/new_types.html
[2]: https://docs.rs/surrealdb/latest/surrealdb/sql/struct.Thing.html/
[3]: https://github.com/ulid/spec
