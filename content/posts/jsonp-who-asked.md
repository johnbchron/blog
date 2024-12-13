---
title: "jsonp - Who Asked?"
written_on: "2024.11.23"
public: true
---

Often I need to prettify JSON.
I probably don't need to justify that need to you.
For many years I have used the site [jsonprettify.com](https://jsonprettify.com), which does the job well.
It doesn't look quite how I would have it look, but it's fit for purpose and allows formatting at different levels of expansion.

Recently, I (mistakenly) thought it had gone down.
This was most likely due to me misremembering the URL on a new machine that didn't remember it for me, but I'm in too deep now to back out anyways.

Finding my favorite tool missing, there's only one real solution.
Build it myself.

## Down the hole

The construction started off pretty simple.
I have been using Rust for the web for the last couple years, and though it changes frequently, I have found that temporarily forsaking JS has made it impossible to pick up again, so here we are.
Leptos has been my tool of choice, and since I'm a Nix stan, I package it with Nix.
All this I have been able to borrow from previous projects of mine.

The main functionality wasn't so difficult either.
The only hurdle is trying not to perform too many copies and not to skip frames when dealing with large payloads.
I found that the HTML `<textarea>` tag produces far more latency than my code when receiving a large payload anyways (about 12 frames at 60fps compared to my 3-4 frames), so I'm not too concerned about it.

`v0.1` was all well and good until I decided to deploy it.
My tradition has been to build it into an OCI image using Nix and then ship it off to [Fly.io](https://fly.io) in a GitHub action, but this time I hesitated.
I have been wanting to move to [CloudFlare Workers](https://workers.cloudflare.com/) as my primary Leptos deployment strategy, since it's free until 100,000 requests per month, and it fits most of my apps.

## Wrangling `wrangler`
`wrangler` is a pain in my ass.
It's the CF workers et al. deployment tool, and it's a JS app.
It doesn't seem like there's a community effort to circumvent it at all.
It itself is fine, but it's packaged poorly within Nix.
On `aarch64-linux` (hello to my Asahi brothers), it doesn't build, complaining something about MUSL `libc.so.1` being missing.

After 15 wasted hours, I dropped it for a week.
After a week, 10 minutes and 2 line additions fixed the problem.

I need to do more research on the `wrangler` build process so that I can squish it down into the Nix ecosystem and forget about it, but I'm satisfied for now.
I need to let my executive energy regenerate before I return to this problem again.

## It works!
This project -- dubbed `jsonp` but unrelated to the weird pre-CORS JS technique [JSONP](https://en.wikipedia.org/wiki/JSONP) -- is now live at [`https://jsonp.org/`](https://jsonp.org/).
Feel free to use it, and direct your feedback to the [GitHub issues page](https://github.com/johnbchron/jsonp/issues).
You can see the planned features there.

Thanks for reading!
