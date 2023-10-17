mod chat;
mod encryption;
mod keys;
mod message;
mod sign_in;
mod state;
mod ws;

use dioxus::prelude::*;
use dioxus_router::prelude::*;
use log::LevelFilter;

use chat::ChatPage;
use serde::{Deserialize, Serialize};
use shared::domain::RoomId;
use sign_in::SignInPage;

use crate::keys::KeysPage;

fn main() {
    dioxus_logger::init(LevelFilter::Info).expect("failed to init logger");
    console_error_panic_hook::set_once();

    log::info!("starting app");
    dioxus_web::launch(app);
}

#[derive(Debug, Default)]
pub struct AppState {
    pub access_toked: Option<String>,
    pub refresh_toked: Option<String>,
}

fn app(cx: Scope) -> Element {
    use_shared_state_provider(cx, AppState::default);

    render! { Router::<Route> {} }
}

// 1. Declare our app's routes
#[derive(Clone, Routable)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/signin")]
    SignInPage {},
    #[route("/:room_id")]
    ChatPage { room_id: String },
    #[route("/:..route")]
    PageNotFound { route: Vec<String> },
}

fn Home(cx: Scope) -> Element {
    render! { h1 { "Welcome to the Dioxus Blog!" } }
}

#[inline_props]
fn PageNotFound(cx: Scope, route: Vec<String>) -> Element {
    render! {
        h1 { "Page not found" }
        p { "We are terribly sorry, but the page you requested doesn't exist." }
        pre { color: "red", "log:\nattemped to navigate to: {route:?}" }
    }
}
