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

pub(crate) fn page_wrapper(children: Markup, ctx: Ctx) -> Markup {
  let stylesheet = &ctx.state().stylesheet;

  let preload_fonts = PRELOAD_FONT_PATHS.iter().map(|p| html! {
    link rel="preload" href=(p) as="font" type="font/woff2" crossorigin="anonymous";
  });

  html! {
    (maud::DOCTYPE)
    html lang="en" {
      head {
        meta charset="utf-8";
        meta name="viewport" content="width=device-width, initial-scale=1";

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

        // font stylesheets
        style { (PreEscaped(include_css!("../../style/fonts/roboto.css"))) }

        title { "John Lewis" }

        // icon
        link rel="icon" type="image/svg+xml" href=(FAVICON_SVG_HREF);
      }
      body {
        main class="m-4 sm:container sm:mx-auto flex flex-col min-h-svh" {
          (children)
          div class="flex-1" {}
        }
      }
    }
  }
}
