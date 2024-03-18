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

---

[1]: https://surrealdb.com/
[^1]: Python definitely beats us with "pythonic", but I think "rusty" is more endearing.
