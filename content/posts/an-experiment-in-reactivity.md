---
title: "An Experiment in Reactivity"
written_on: "2024.12.15"
public: false
---

A few weeks ago I was working a conference and I needed a distraction to let my brain unclench when I was off-duty.
Naturally the activity I chose ended up being a difficult programming project.

I'm planning to build/building a novel animation framework in Rust, and one of the goals I have for that project is to allow temporal reactivity.
Like a normal reactive system, in a temporally reactive system there are a set of signals which can each depend on the evaluation of other signals, forming a DAG.
Unlike a normal reactive system, there is an **evaluation period** and a **step size**, so each signal is evaluated at a set of discrete steps.
Signals can also depend on the evaluation of other signals *in the past or future*.

Seems difficult, huh?
I thought so, especially because I haven't built any sort of reactive system before.
I also don't have any formal education background which would be helpful here, and I've never even heard of this kind of thing before.

So I decided to do a practice run; a normal reactive system that is only evaluated at a single point in time.
This post is a discussion of that attempt.
Keep in mind I've never done this before.
Some inspiration is taken from [Leptos](https://www.leptos.dev/).

## Signal Matrix Composition

The first "design principle" I thought of was to make a lightweight `Signal` type, which would be an identifier for the heavier `SignalDef`.

`Signal` is actually quite drab:

```rust
/// A handle to a signal in the graph. This is an ID for a signal definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Signal(u64);
```

It gets the job done. It's `Copy` so you can throw it around easily.

I decided to make the system generic on `SignalDef`, so that it has the potential to be useful.
So `SignalDef` is actually a trait.
It looks like this:

```rust
/// Trait for signal definitions.
pub trait SignalDef: Debug + Sync + Sized {
  /// The type of value that this signal definition evaluates to.
  type Value: Debug + Send + Sync + Sized;

  /// Get the dependencies of this signal definition.
  fn dependencies(&self) -> HashSet<Signal>;
  /// Evaluate this signal definition with the given context.
  fn evaluate(&self, ctx: &EvalContext<Self>) -> Self::Value;
}
```

There are a couple of interesting things in here.

Firstly, `Value` is an associated type, o the `SignalDef` implementer chooses the type of its evaluation result; interesting.

Secondly, each `SignalDef` implementer must be able to barf up a set of its dependencies.
And I'm enforcing this strictly.
During evaluation time, it will only have access to those signals, and it cannot change its dependency set based on the values of the dependency set it's already returned.
This keeps things nice and tight, and opens up our options for solver algorithms.

Thirdly, the `evaluate()` function takes an `EvalContext<Self>`, which holds the values of the signals it asked for in its dependency array.

This is the definition for `EvalContext`:

```rust
/// Context given to an evaluator function. For providing dependencies.
pub struct EvalContext<'c, T: SignalDef> {
  values: HashMap<Signal, &'c T::Value>,
}
```

Pretty simple.
It has a named lifetime because it will borrow values from signals already evaluated.
Note that `SignalDef::Value` doesn't require `Clone`; we're building this to allow huge value types because we can.

## Planning Evaluations

I decided to add a "planning" phase before evaluating so that we never unnecessarily compute a value.
Doing so adds more overhead, but that's fine because I'm gearing this toward more heavy calculations anyways.

The input to the planning operation is a set of targets, indicating which signals we're asking to get the values of.
None of the other signals *have* to be evaluated - only the ones that our targets depend on.

Since we're operating on a DAG, we can organize the signals we need to evaluate into a series of phases.
Each phase will only contain signals whose dependencies have been evaluated in previous phases.
The first phase will only have signals that have no dependencies.

### Bad Approaches

I realize now in retrospect, while writing this blog post, that I could have described this problem using graph theory and used known algorithms to plan the phases.
I don't know graph theory, so I didn't think of this while writing my solution to the planning algorithm.

I'll show a visual example of what I'm talking about.

![](/svgs/reactivity-1.svg)

Here the `SignalDef` type we're using is `FloatSignalDef`, made for calculating `f64`s.

```rust
/// A signal definition for a floating-point value or operation.
#[derive(Debug)]
pub enum FloatSignalDef {
  Constant(f64),
  UnaryOp(UnaryOp),
  BinaryOp(FloatBinaryOp),
}

/// A binary operation on floating-point signals.
#[derive(Debug)]
pub enum FloatBinaryOp {
  Add(Signal, Signal),
  Sub(Signal, Signal),
  Mul(Signal, Signal),
  Div(Signal, Signal),
  Pow(Signal, Signal),
}

/// A unary operation on a floating-point signal.
#[derive(Debug)]
pub enum UnaryOp {
  Neg(Signal),
}

impl SignalDef for FloatSignalDef {
  type Value = f64;

  fn dependencies(&self) -> HashSet<Signal> {
    match self {
      FloatSignalDef::Constant(_) => HashSet::new(),
      FloatSignalDef::UnaryOp(op) => match op {
        UnaryOp::Neg(s) => vec![*s].into_iter().collect(),
      },
      FloatSignalDef::BinaryOp(op) => match op {
        FloatBinaryOp::Add(a, b) => vec![*a, *b].into_iter().collect(),
        FloatBinaryOp::Sub(a, b) => vec![*a, *b].into_iter().collect(),
        FloatBinaryOp::Mul(a, b) => vec![*a, *b].into_iter().collect(),
        FloatBinaryOp::Div(a, b) => vec![*a, *b].into_iter().collect(),
        FloatBinaryOp::Pow(a, b) => vec![*a, *b].into_iter().collect(),
      },
    }
  }

  fn evaluate(&self, ctx: &EvalContext<Self>) -> Self::Value {
    match self {
      FloatSignalDef::Constant(value) => *value,
      FloatSignalDef::UnaryOp(op) => match op {
        UnaryOp::Neg(s) => -ctx.values[s],
      },
      FloatSignalDef::BinaryOp(op) => match op {
        FloatBinaryOp::Add(a, b) => ctx.values[a] + ctx.values[b],
        FloatBinaryOp::Sub(a, b) => ctx.values[a] - ctx.values[b],
        FloatBinaryOp::Mul(a, b) => ctx.values[a] * ctx.values[b],
        FloatBinaryOp::Div(a, b) => ctx.values[a] / ctx.values[b],
        FloatBinaryOp::Pow(a, b) => ctx.values[a].powf(*ctx.values[b]),
      },
    }
  }
}
```
