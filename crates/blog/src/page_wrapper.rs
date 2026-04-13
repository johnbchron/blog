use css_minify_macro::include_css;
use maud::{Markup, PreEscaped, html};

use crate::ctx::Ctx;

const PRELOAD_FONT_PATHS: &[&str] = &[];
const FAVICON_SVG: &str = include_str!("../../../public/favicon.svg");
const FAVICON_SVG_BASE64: &str =
  const_base::encode_as_str!(FAVICON_SVG, const_base::Config::B64);
const FAVICON_SVG_HREF: &str =
  const_format::concatcp!("data:image/svg+xml;base64,", FAVICON_SVG_BASE64);
const HTMX_ASSET_PATH: &str = "/dist/htmx.min.js";
const HTXM_CONFIG: &str = r#"{ "globalViewTransitions": true }"#;

pub(crate) fn page_wrapper(
  page_title: impl AsRef<str>,
  children: Markup,
  ctx: Ctx,
) -> Markup {
  const HEAD_TITLE_PREFIX: &str = "John Lewis";

  let stylesheet = &ctx.state().stylesheet;

  let preload_fonts = PRELOAD_FONT_PATHS.iter().map(|p| html! {
    link rel="preload" href=(p) as="font" type="font/woff2" crossorigin="anonymous";
  });

  let page_title = page_title.as_ref();
  let head_title = if page_title == HEAD_TITLE_PREFIX {
    HEAD_TITLE_PREFIX.to_owned()
  } else {
    format!("{HEAD_TITLE_PREFIX} - {page_title}")
  };

  html! {
    (maud::DOCTYPE)
    html lang="en" {
      head {
        meta charset="utf-8";
        meta name="viewport" content="width=device-width, initial-scale=1";
        meta name="title" content=(head_title);
        meta name="description" content=(crate::home_page::SITE_DESCRIPTION);

        // load htmx
        script src=(HTMX_ASSET_PATH) { }
        meta name="htmx-config" content=(HTXM_CONFIG);

        // include columbo swap script
        script { (PreEscaped(columbo::GLOBAL_SCRIPT_CONTENTS)) }

        // preload fonts
        @for preload_font in preload_fonts {
          (preload_font)
        }

        // main stylesheet
        style { (PreEscaped(stylesheet)) }

        // arborium highlighting
        link rel="stylesheet" href="/dist/base-arborium.css";
        link rel="stylesheet" href="/dist/cattpuccin-mocha.css";
        link rel="stylesheet" href="/dist/cattpuccin-latte.css";

        // font stylesheets
        style { (PreEscaped(include_css!("../../style/fonts/ibm_plex_serif.css"))) }

        title { (head_title) }

        // icon
        link rel="icon" type="image/svg+xml" href=(FAVICON_SVG_HREF);
      }
      body class="bg-light-bg-1 dark:bg-dark-bg-1 text-light-fg dark:text-dark-fg font-serif" {
        main class="p-4 sm:container sm:mx-auto flex flex-col gap-4 min-h-svh" {
          div class="flex flex-row gap-2 items-center text-light-fg-dim dark:text-dark-fg-dim text-sm" {
            a href="/" class="link" { "Home" }
            "•"
            a href="/about" class="link" { "About" }
            "•"
            a href="/posts" class="link" { "Posts" }
          }
          div class="markdown" {
            p class="title" { (page_title) }
            (children)
          }
          div class="flex-1" {}
          div class="h-0 border-t border-light-fg-dimmer dark:border-dark-fg-dimmer" {}
          div class="w-full flex flex-row justify-end" {
            p class="text-sm text-light-fg-dimmer dark:text-dark-fg-dimmer" {
              "Thanks for reading."
            }
          }
        }
      }
    }
  }
}
