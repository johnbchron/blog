---
title: "A New TiKV"
written_on: "2024.12.13"
public: true
---

I *kind of* like [TiKV](https://tikv.org/).

# What I've grown to expect

I'm building an app -- watch me build it in the open [here](https://github.com/rambit-systems/rambit) -- for which I've chosen to base the data model around a key-value store.
The flavor of KV store I'm building it to expect has the following basic characteristics:

1. Keys and values are plain bytes, and methods don't try to interpret my data.
Leave understanding my data to me, plz and thank you.
2. Simple methods.
`get`, `put`, `delete`.
Add `insert` if you care about that sort of thing; bulk methods if you want.
Not much more.

In addition, I need a transactional interface so that I can enforce data integrity.
Some people are fine with inconsistent data or with doing all the extra operational validation themselves, but I am not.
There's a great writeup by `foundationdb` about transactions called the [Transaction Manifesto](https://apple.github.io/foundationdb/transaction-manifesto.html).

Transactions come in two flavors: `optimistic` and `pessimistic`.
If you don't know the difference, it boils down to the following.
If there would be a collision between two transactions, `pessimistic` transactions try to force themselves through, and `optimistic` transactions back off.
They're meant for different use cases.
If you think the data you're trying to modify is contested, use a `pessimistic` transaction.
If not, use an `optimistic` transaction and save some resources.

Finally, it needs to be distributed, because I need to scale.

---

I originally modeled this interface around what TiKV provides, because I think it's a good standard.
It's pretty unopinionated, and easy to build other data models on top of.

I decided to use this set of semantics for my data store "interface".
My app doesn't actually care what I'm using, so long as it can be squished into this paradigm (thanks to [this great piece of literature](https://www.howtocodeit.com/articles/master-hexagonal-architecture-rust)).

# What to use?

Though Redis is the most commonly known key-value store, it's definitely not a candidate.
It really wants to be in charge of your data, and it treats you as though you don't know how to serialize and deserialize.
It lacks real transactions, and is certainly not scalable.
It's a great tool, just not for this use-case.

So let's turn to the heavyweights: FoundationDB and TiKV.

In principle, they're both great.
They're highly scalable, they have ACID-compliant transactions, they're data-agnostic.
I know the most about TiKV because it's what I started using for this project.
They're battle tested and production ready.

So why am I writing this blog post.

FoundationDB and TiKV are just **too dang big**.

# The problem with TiKV

I'm going to just talk about TiKV now, because I haven't experimented with FoundationDB, but most of this will still be applicable.

TiKV is too big.
It's just really heavy.

It uses RocksDB as its shard implementation, and while that's fine, we now have much better modern Rust alternatives.
RocksDB is pretty stable, but because it's written in C++, it's a pain in the ass to interface with.

TiKV also really suffers from "kubernetes bloat".
It requires a completely separate component called `PD` just to manage keyspace shards and meshing.
In concept, this is completely unnecessary, and TiKV nodes should simply form a lattice.
I know there's nothing "simple" about a lattice, but this is my complaint post and I can say what I want.

It just feels huge.
There's no single-node deployment, even for testing, at least not without using the weird TiUP playground or whatever.
The memory footprint is too high.
The list goes on.

# Solution?

I'm just going to pout and say "I'll make my own".
[Hubris will be the motivator](https://youtube.com/clip/UgkxKTbotBTSMANrZAqualPrwBShDL3KFagZ?si=hmO_YOnPWwEgCARc) for this project, and I will simply succeed, just like we all expect.

Some quick design principles:

1. **Rust.** For obvious reasons.
2. **"Small", accessible codebase.**
I don't want this to become impenetrable with time; crates should be well-purposed, efficient, well tested, etc.
3. **Lightweight.**
The memory footprint should be small and closely coupled with 1) the amount of data being stored in a node, and 2) the amount of cooperation the node is participating in.
4. **A single component.**
I don't want a `PD` component here.
Coordination will be managed by nodes themselves, and I will steal the ["lattice"](https://wasmcloud.com/docs/concepts/lattice) architecture from [wasmCloud](https://wasmcloud.com/), I think.

---

This is a rant, but I may very well just go off and do it myself.
I really am surprised that there are so few distributed transactional key-value stores.
They are hard to build, of this I am certain, but we could use some more market variety.
