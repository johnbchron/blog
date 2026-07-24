+++
title = "Feeling Lucky with DuckDuckGo"
date = "2026-04-27"
+++

I learned today by accident that DuckDuckGo has an "I'm Feeling Lucky" feature.

I was searching on DuckDuckGo with the search term `\uagd` for *reasons*, and to
my surprise, it sent me straight to the website for the [University of Arizona's
global campus](https://myuagc.uagc.edu/). I tried it again and it took me a
second to realize what was happening. Looking at the search results after having
removed the backslash made it pretty clear that it was redirecting me to the
first result it found.

Naturally my next question is, how does it work? I'm a curious fellow.

Is it simply returning a status code? SEO is the business of the search engine,
and every response code carries a distinct meaning, so a simple `30x` status
wouldn't cut it. How would a search engine handle such an "administrative
redirect"?

This redirect isn't one that should be remembered by browsers or consumed by
other search engines, rather only executed by the browser. So how does it work?

## The Internals

In truth, it's delightfully simple. When I request a search like
`\docs rs columbo` ([`columbo`](https://docs.rs/columbo) is a minimal
streaming-suspense library I wrote recently), my browser encodes that as
`https://duckduckgo.com/?q=%5C+docs+rs+columbo&t=ftsa&ia=web`. To be honest I
don't quite know what the `t=ftsa` and `ia=web` mean, but I assume they're some
sort of agent/origin profiling. Let's see what that gives us through `curl`:

```sh
curl 'https://duckduckgo.com/?q=%5C+docs+rs+columbo&t=ftsa&ia=web'
```

DuckDuckGo happily responds with an HTTP/2 `200` response and concise HTML:

```html
<html>
  <head>
    <meta http-equiv="Content-Type" content="text/html; charset=utf-8">
    <meta name="referrer" content="origin">
    <meta name="robots" content="noindex, nofollow">
    <meta http-equiv="refresh" content="0; url=/l/?uddg=https%3A%2F%2Fdocs.rs%2Fcolumbo%2Flatest%2Fcolumbo%2F&amp;rut=f1de9ec9c679ac80683c46eb729e7dc3711ee91e230a29da75f089568bcc2023">
  </head>
  <body>
    <script language="JavaScript">function ffredirect(){window.location.replace('/l/?uddg=https%3A%2F%2Fdocs.rs%2Fcolumbo%2Flatest%2Fcolumbo%2F&rut=f1de9ec9c679ac80683c46eb729e7dc3711ee91e230a29da75f089568bcc2023');}setTimeout('ffredirect()',100);</script>
  </body>
</html>
```

The `referrer` and `robots` meta tags both make sense, but I wasn't aware of the
`refresh` header until I was researching this. It indicates to the browser to
redirect even if JavaScript is disabled, but of course there's that JS snippet
in the body as well.

What I'm interested in primarily is the link that it's sending me to:

```
/l/?uddg=https%3A%2F%2Fdocs.rs%2Fcolumbo%2Flatest%2Fcolumbo%2F&rut=f1de9ec9c679ac80683c46eb729e7dc3711ee91e230a29da75f089568bcc2023
```

It points to `https://duckduckgo.com/l` with the following query parameters:

```toml
uddg = "https://docs.rs/columbo/latest/columbo/"
rut = "f1de9ec9c679ac80683c46eb729e7dc3711ee91e230a29da75f089568bcc2023"
```

I suppose `rut` is just a request identifier.

If we visit that URL (ending in `/l`) we get something similar:

```html
<html>
  <head>
    <meta name="referrer" content="origin">
  </head>
  <body>
    <script language="JavaScript">window.parent.location.replace("https://docs.rs/columbo/latest/columbo/");</script>
    <noscript>
      <meta http-equiv="refresh" content="0;URL=https://docs.rs/columbo/latest/columbo/">
    </noscript>
  </body>
</html>
```

No clue why the meta tag here is in the body with a `<noscript>`, but this is
dead simple.

I'm quite delighted by the simplicity here, and by the lack of tracking data
other than the simple identifier. I checked the headers for both of these
requests and it was nice and clean.

Thank you DuckDuckGo for existing!

Dear reader, if you made it this far, I don't know why you're not bored yet, but
thanks for reading.
