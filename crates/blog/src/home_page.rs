use axum::response::IntoResponse;
use maud::html;

use crate::{ctx::ResponseSeed, page_wrapper::page_wrapper};

pub(crate) async fn home_page(
  ResponseSeed(ctx, resp): ResponseSeed,
) -> impl IntoResponse {
  let page = html! {
    div class="flex flex-col" {
      p class="title" { "Hello, World!" }
      p {
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vestibulum nec arcu vitae enim ullamcorper imperdiet. Quisque sed nibh nulla. Nunc lacinia non felis et posuere. Morbi fringilla, nunc a consectetur placerat, tellus ligula fermentum diam, ac vehicula eros leo et ipsum. Donec vitae libero ut neque semper mollis a eget mauris. Vestibulum arcu erat, fermentum id ligula sit amet, placerat rhoncus est. Integer iaculis risus ut luctus eleifend. Aliquam nibh neque, aliquet sit amet arcu et, tempus vestibulum quam. Aenean condimentum eros nulla. Donec sit amet finibus lacus, vel fringilla elit. Nulla feugiat ullamcorper lorem in ornare. Pellentesque a ex et nisl aliquam commodo. Aenean a augue viverra, ullamcorper dolor a, mollis augue. Morbi molestie dui at libero vehicula placerat."
      }
    }
  };

  let doc = page_wrapper(page, ctx);
  resp.into_stream(doc)
}
