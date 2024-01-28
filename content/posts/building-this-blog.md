---
title = "Building this blog: A Space Odyssey"
written_on = "28/01/24"
posted = true
---

# Building this blog: A Space Odyssey

### Preamble
Welcome to my blog. This poor thing is a work of love, handcrafted to my tastes. I've heard it said that writing a blog is like the `Hello World!` of web development, i.e. the least useful application, and after building one myself, I agree. If I only wanted a blog, there are already plenty of competent solutions. I could go the consumer route with Wix or Wordpress, or if I wanted to keep my soul with me I could choose a more philosophically-pleasing developer-oriented option, or even just a static-site generator.

I'm not building this just to have a blog though; like the `Hello World!` program, it's an exercise. A blog is also a deeply personal item, thus there's value in making it your own.

### Initial Thoughts
When I was beginning this project, I knew I wanted to write it in Rust, as with all things I build these days. I also wanted to build it with Nix, due to its [many benefits](https://jonathanlorimer.dev/posts/nix-thesis.html), and because it's something I want to gain more experience with.

As with any language, web development requires a better answer than just "in Rust". In the JS world, we're surrounded with towering and lumbering web frameworks that each have their exclusive benefits and abstract over mostly the same portions of the "full stack" grand canyon. Rust has fewer and more diverse options. It's time to choose a framework.

As I saw them, the workable options are [Leptos](https://www.leptos.dev/) and [Dioxus](https://dioxuslabs.com/). I decided on Leptos because I've worked with it before and it seems a bit more feature-complete. Dioxus also markets itself more as a a GUI framework, so I thought Leptos' direction matched my needs better. Another difference that's notable but not super valuable to me is reactive style; Dioxus is like React and Leptos is like Solid.

Some other choices I made were to use [TailwindCSS](https://tailwindcss.com/) for styling and Markdown for post content. I also decided to use [fly.io](https://fly.io/) for hosting, which meant the server would need to run in a Docker (read: [OCI](https://github.com/opencontainers/runtime-spec/blob/main/spec.md)) container.

### Getting Started
I used the Leptos [`start-axum-workspace`](https://github.com/leptos-rs/start-axum-workspace) to seed the repo because the last time I used Leptos, the repository seemed very cluttered with too many `#[cfg(...)]` directives and feature flags, and it looked like it would benefit from being separated into multiple crates. I was correct.

The first thing I noticed was that the dependencies were being specified in an odd way. They were being declared in the workspace `Cargo.toml` file and then subscribed to in the individual crate `Cargo.toml` files:
```root/Cargo.toml
[workspace.dependencies]
leptos = { version = "0.5", features = ["nightly"] }
leptos_meta = { version = "0.5", features = ["nightly"] }
leptos_router = { version = "0.5", features = ["nightly"] }
leptos_axum = { version = "0.5" }
```

```root/app/Cargo.toml
[dependencies]
leptos.workspace = true
leptos_meta.workspace = true
leptos_router.workspace = true
leptos_axum = { workspace = true, optional = true }
```

Apparently this is valid. It seems like a decent way to explicitly make sure that your dependencies are locked between packages. Noted; moving on.

### Leptos Primitives
I'll go over some of the core Leptos concepts to bring everyone up to speed.

Leptos uses a signal-based reactivity approach, similar to [Solid.JS](https://www.solidjs.com/). I won't go over the specifics of signals -- you can read about them [here]() -- but they are being increasingly used as the lightweight, headache-less, next-generation alternative to typical React-style hooks.

Components in Leptos are functions which return `impl IntoView` and are annotated with `#[component]`. The component is called once to render, and anything that will change is contained within a closure. During the the render of a component or closure, Leptos tracks what signals the component/closure uses. When a tracked signal changes, the components and closures that use that signal will be re-rendered, making targeted updates to the DOM.

Typically when using server-side rendering (SSR) in Leptos, all components have to be able to run on the server and the client. This allows us to render the whole page on the server for the first visit, and then render on the client when the user navigates or interacts with the page.

#### Rust Detour
Leptos uses a Rust macro -- `view! { ... }` -- that allows the developer to write HTML-like markup as the bulk of component code, with conventions similar to JSX/TSX. Being able to do this without any preprocessors and to evaluate down to Rust syntax at compile-time is an incredible feat, and I would sing the praises of Rust macros here but others have [done it for me](https://www.youtube.com/watch?v=MWRPYBoCEaY).

### Islands
Islands are a relatively new feature in Leptos, and are useful in this static-site-adjacent blog. The name refers to having "islands" of client-hydrated interactivity within a "sea" of server-rendered HTML. By adding the `experimental-islands` feature and making a change in the frontend code, we can mandate that `#[component]` components are always run only on the server (the sea), and `#[island]` components get hydrated in isolation.

This is important for a couple of reasons. Firstly, I can call server-only code within `#[component]` components without the usual abstractions that are necessary for writing co-rendered components (renderable on both client and server). Secondly, I can do this without resorting to a fully static approach, which would disbar me from using SSR features like reading from a database.

### Markdown
When looking for Markdown parsers, I found the mature [`pulldown-cmark`](https://crates.io/crates/pulldown-cmark). It seemed sufficient and uses a novel pull-parser, producing an event stream instead of an AST as its internal representation. Its `Parser` type is an iterator over events, and tranformations are possible by modifying the event list before rendering it to HTML.

Let's see how it works.
```rust
fn get_markdown_content(path: String) -> String {
  let path = format!("./content/{path}");
  let mut file = std::fs::File::open(&path).expect("failed to open file");
  let mut input = String::new();
  file
    .read_to_string(&mut input)
    .expect("failed to read file");

  let parser =
    pulldown_cmark::Parser::new_ext(&input, pulldown_cmark::Options::all());
  let mut html_output = String::new();
  pulldown_cmark::html::push_html(&mut html_output, parser);

  html_output
}
```
```rust
#[component]
fn Markdown(
  #[prop(into)] path: String,
  #[prop(into, default = String::new())] class: String,
) -> impl IntoView {
  let content = get_markdown_content(path);
  view! {
    <div class=format!("markdown {class}")>{html::div().inner_html(content)}</div>
  }
}
```

`get_markdown_content()` runs on the server and, given a `String` path relative to my `/content` directory, returns a `String` with the rendered HTML. I can transparently render the HTML within a `<div>` to use it in the `Markdown` component.

The eagle-eyed among you might notice that `Markdown()` is a capitalized Rust function. That's normally a no-no, but it's helpful within the context of Leptos to allow distinguishing easily between functions that act as components and regular functions. We can satisfy Clippy with a crate-level `#![allow(non_snake_case)]` directive.

### Adding Heading Anchors
`pulldown-cmark` has no built-in solution for adding id-based anchors to headings, but that's something I want, so that you, the user, can send your friends links to specific headers. Aren't I so generous?

There's two parts to this; adding HTML IDs to the headings, and then adding the `<a>` tags within each of those headings.

The following function loops through the event list produced by the `Parser` and tracks when we pass a heading opening tag, accumulates the `Event::Text` within the heading, and adds the ID and anchor when we exit the heading. The pull-parser is what enables this sort of linear-tape traversal. The pull-parser architecture is growing on me.

We're using the [`slug`](https://crates.io/crates/slug) crate to convert the text inside the heading into a url-safe slug. The `slug` crate reminds of why I love Rust; what a wonderful library.

```rust
fn add_markdown_heading_ids(events: Vec<Event<'_>>) -> Vec<Event<'_>> {
  let mut parsing_header = false;
  let mut heading_id = String::new();
  let mut events_to_return = Vec::new();

  for event in events {
    match event {
      Event::Start(pulldown_cmark::Tag::Heading(_, _, _)) => {
        parsing_header = true;
        heading_id.clear();
      }
      Event::End(pulldown_cmark::Tag::Heading(_, _, _)) => {
        parsing_header = false;
        heading_id = slug::slugify(heading_id.as_str());

        events_to_return.push(Event::Text(CowStr::from(" ")));
        events_to_return.push(Event::Html(CowStr::from(format!(
          "<a href=\"#{}\" id=\"{}\"><span class=\"anchor-icon\">#</span></a>",
          heading_id, heading_id
        ))));
      }
      Event::Text(ref text) => {
        if parsing_header {
          heading_id.push_str(text);
        }
      }
      _ => {}
    }
    events_to_return.push(event);
  }

  events_to_return
}
```

This isn't very pretty, I know, but it's sufficient. We can use our event-stream-modifying function in `get_markdown_content()`: 

```rust
  let events = add_markdown_heading_ids(parser.into_iter().collect());
  let mut html_output = String::new();
  pulldown_cmark::html::push_html(&mut html_output, events.into_iter());
```

### Styling
With a little bit of (admittedly finicky) package meta configuration, I was able to get Leptos' built-in Tailwind support working. This meant that the site styling was nearly trivial, but styling the markdown posed a greater challenge.

Ordinarily, one would use the `@tailwindcss/typography` first-party Tailwind plugin, but it didn't have the look that I wanted out of the box, and after 20 minutes of fiddling I decided to try a different approach.

I copied the styling from [here](https://bevyengine.org/news/sme-announcements/) straight into my `main.scss` and immediately found out that Leptos will only compile Tailwind or Dart Sass, but [not both](https://github.com/leptos-rs/cargo-leptos/issues/209), because Tailwind apparently doesn't like preprocessors, so I set about porting my Sass in to CSS.

Even when writing CSS though, Tailwind provides tangible benefits. Take for example the following CSS:

```css
.markdown code {
    margin-left: 0.125rem;
    margin-right: 0.125rem;
    white-space: nowrap;
    border-radius: 0.25rem;
    border-width: 1px;
    border-color: rgb(82 82 91 / 1);
    background-color: rgb(39 39 42 / 1);
    padding-left: 0.375rem;
    padding-right: 0.375rem;
    padding-top: 0.125rem;
    padding-bottom: 0.125rem;
    font-family: Iosevka Term Web, ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, Liberation Mono, Courier New, monospace;
    font-size: 1rem;
    line-height: 1.5rem;
    font-variant-numeric: normal;
}
```

Disgusting. I mean, it's not terrible by itself, but CSS's failure to scale properly and follow basic programming precepts such as reusability, loose coupling, etc. is why we have 1,000,001 CSS preprocessors. My disgust comes not from what I see in front of me, but what I see ahead of me. Maintaining bare CSS is a nightmare.

We can't fix the separation of concerns and loose coupling here, but we *can* just be careful about how we pick our selectors within the limited scope of Markdown styling.

The other half of failing to scale is in readability and reusability. Working with all these bare numbers is bad practice. We need semantically correct names for the values -- for example separating padding constants from line-height constants -- and we need a system to make sure that the values are only applied to the properties to which they actually apply.

This is where Tailwind's `@apply` comes in. We can refactor the above code to:
```css
.markdown code {
  @apply text-base font-mono normal-nums bg-zinc-800 rounded border border-zinc-600 px-1.5 py-0.5 mx-0.5 whitespace-nowrap;
}
```
I breathe a sigh of relief, but the moment is brief.

### Deployment & CI/CD

#### The Problem

Unknowingly, I recently incorporated an item into my workflow that would be a major stumbling block in this project -- my new MacBook Pro M2 Pro (they really did put "Pro" in twice).

I wanted to deploy to [fly.io](https://fly.io/), which requires Docker images, like most hosting services. Fly.io only leases `x86_64` compute. The problem here is that the architecture of my MacBook is `aarch64`, not `x86_64`.

#### The Solution

I won't drag you through the slog of trial and error that it took to overcome the effort threshold for me to give up on deploying from my machine -- partially because I don't remember most of what I did -- but eventually I decided to just build and deploy the image from Github Actions, where I could request an `x86_64-linux` runner.

The attempts/challenges included attempting to cross-compile in MacOS and digging through the Nix builder logs until I learned that Darwin doesn't support virtualization as a kernel feature, attempting to run a generic QEMU configuration on Asahi Linux and **segfaulting on grep** due to the M2's asymmetric E- and P-cores, discovering all sorts of quirks relating to how Nix handles cross-compilation, etc. 

If you'd like to look through my `flake.toml` you can find it [here](https://github.com/johnbchron/blog/blob/main/flake.nix). It's reasonably clean, and I used [Crane](https://crane.dev/index.html) to build the Rust, and then the native docker tools to make the image. From there I just have to have the secret `FLY_API_TOKEN` and I can push to fly.

### Future Work

This blog works, but there are some more features that I'd like to have.

The first is syntax highlighting. I refuse to just throw a JS snippet in there to load highlighting styles after the page load, mainly just because I've made it this far on pure Rust. The `syntect` library seems to be the best choice, and I'll implement it the same way by iterating through the markdown `Event`s, I just haven't done it yet.

The other is using a proc-macro to manage posts. I'd love to have a proc macro just scan a list of post files at compile time and automagically build the route list and post list for wherever in the project I need them, and it seems that this would be a nice introductory project for proc macros. I've build declarative `#[derive()]` macros and simple `macro_rules!()` proc macros for syntax sugar, but nothing that uses the file system at compile time.

### Conclusion

Thanks for sticking with me this far. Really, thank you. I hope you enjoyed this post or at least learned something. This was my first blog post ever, so it's special to me.

If you have any comments or questions, please feel free to [email me](mailto:blog@jlewis.sh); I'd love to hear your feedback. Cheers!
