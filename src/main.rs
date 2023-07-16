use leptos::*;

#[component]
fn App(cx: Scope) -> impl IntoView {
    let (count, set_count) = create_signal(cx, 0);

    view! { cx, 
        <button on:click=move |_| {
            set_count(3);
        }>
        "Click Me: "
            {move || count.get()}
        </button>
    }
}

fn main() {
    mount_to_body(|cx| view! { cx, <App/> })
}
