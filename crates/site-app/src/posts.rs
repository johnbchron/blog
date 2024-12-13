#[cfg(feature = "ssr")]
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
  pub path:         String,
  pub metadata:     PostMetadata,
}

impl Post {
  pub fn full_post(&self) -> impl IntoView {
    leptos::leptos_dom::html::div()
      .attr("class", "markdown")
      .inner_html(self.html_content.clone())
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostMetadata {
  pub title:      String,
  pub written_on: String,
  pub public:     bool,
}

#[cfg(feature = "ssr")]
pub fn extract_post(path: &str, input: &str) -> Post {
  let matter = Matter::<YAML>::new().parse(input);
  let metadata = matter.data.unwrap();

  Post {
    html_content: crate::markdown::markdown_to_html(&matter.content),
    path:         path.to_string(),
    metadata:     PostMetadata {
      title:      metadata["title"].as_string().unwrap(),
      written_on: metadata["written_on"].as_string().unwrap(),
      public:     metadata["public"].as_bool().unwrap(),
    },
  }
}

#[server]
pub async fn get_all_posts() -> Result<Vec<Post>, ServerFnError> {
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

      posts.push(extract_post(
        path.file_stem().unwrap().to_str().unwrap(),
        &input,
      ));
    }
  }

  posts.retain(|p| p.metadata.public);
  posts.sort_by(|a, b| {
    // I know this is wrong and only reverses the order of code points, but it's
    // fine for now
    let a = a
      .metadata
      .written_on
      .to_string()
      .chars()
      .rev()
      .collect::<String>();
    let b = b
      .metadata
      .written_on
      .to_string()
      .chars()
      .rev()
      .collect::<String>();
    a.cmp(&b)
  });

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

  let post = extract_post(&path, &input);

  if post.metadata.public {
    Ok(post)
  } else {
    Err(ServerFnError::new("Post not found"))
  }
}

#[component]
pub fn PostPage() -> impl IntoView {
  let params = use_params_map();
  let path = params().get("path").unwrap().clone();

  let post_resource =
    create_blocking_resource(move || path.clone(), get_post_by_path);

  view! {
    <Suspense>
      { move || post_resource.get().map(|p| match p {
        Ok(post) => view! {
          <Title text={post.metadata.title.clone()} />
          <div class="markdown">
            <h1>{post.metadata.title.clone()}</h1>
            <p>Written on {post.metadata.written_on.clone()}</p>
            <hr />
          </div>
          { post.full_post() }
        }.into_view(),
        _ => view! { <p>"Loading..."</p> }.into_view()
      })}
    </Suspense>
  }
}
