use leptos::prelude::*;
use leptos::{component, view, IntoView};
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <main>
                <Routes fallback=|| NotFound>
                    <Route path=path!("") view=HomePage/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <div>
            <h1>"Home Page"</h1>
            <a href="/room/123">"Join Room"</a>
        </div>
    }
}

#[component]
pub fn NotFound() -> impl IntoView {
    view! {
        <div>
            <h1>"404 Not Found"</h1>
        </div>
    }
}