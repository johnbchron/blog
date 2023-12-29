use leptos::*;

#[component]
fn App() -> impl IntoView {
  view! {
    <div class="">
    <main
      class="max-w-2xl mx-auto min-h-screen bg-zinc-800"
    >
    </main>
    </div>
  }
}

fn main() { leptos::mount_to_body(|| view! { <App/> }) }
