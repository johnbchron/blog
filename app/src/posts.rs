use std::io::Read;

#[cfg(feature = "ssr")]
use gray_matter::{engine::YAML, Matter};
use leptos::*;
use leptos_meta::Title;
use leptos_router::use_params_map;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
  pub html_content: String,
  pub metadata:     PostMetadata,
}

impl Post {
  pub fn full_post(&self) -> impl IntoView {
    view! {
      <div class="markdown">{html::div().inner_html(self.html_content.clone())}</div>
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostMetadata {
  pub title:      String,
  pub written_on: String,
  pub public:     bool,
}

#[cfg(feature = "ssr")]
pub fn extract_post(input: &str) -> Post {
  let matter = Matter::<YAML>::new().parse(input);
  let metadata = matter.data.unwrap();

  Post {
    html_content: crate::markdown::markdown_to_html(&matter.content),
    metadata:     PostMetadata {
      title:      metadata["title"].as_string().unwrap(),
      written_on: metadata["written_on"].as_string().unwrap(),
      public:     metadata["public"].as_bool().unwrap(),
    },
  }
}

#[server]
pub async fn get_all_posts() -> Result<Vec<Post>, ServerFnError> {
  // find all files in `content/posts`

  let mut posts = Vec::new();

  for entry in std::fs::read_dir("./content/posts").unwrap() {
    let entry = entry.unwrap();
    let path = entry.path();

    if path.is_file() {
      let mut file = std::fs::File::open(&path).expect("failed to open file");
      let mut input = String::new();
      file
        .read_to_string(&mut input)
        .expect("failed to read file");

      posts.push(extract_post(&input));
    }
  }

  Ok(posts)
}

#[server]
pub async fn get_post_by_path(path: String) -> Result<Post, ServerFnError> {
  let mut file = std::fs::File::open(format!("./content/posts/{}.md", path))
    .expect("failed to open file");
  let mut input = String::new();
  file
    .read_to_string(&mut input)
    .expect("failed to read file");

  Ok(extract_post(&input))
}

#[component]
pub fn PostPage() -> impl IntoView {
  let params = use_params_map();
  let path = params().get("path").unwrap().clone();

  let post_resource = create_blocking_resource(
    move || path.clone(),
    |path| get_post_by_path(path),
  );

  view! {
    <Suspense>
      { move || post_resource.get().map(|p| match p {
        Ok(post) => view! {
          <Title text={post.metadata.title.clone()} />
          { post.full_post() }
        }.into_view(),
        _ => view! { <p>"Loading..."</p> }.into_view()
      })}
    </Suspense>
  }
}
