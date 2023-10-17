use dioxus::prelude::*;

use reqwest::header::CONTENT_TYPE;
use serde::{Deserialize, Serialize};

use crate::{encryption::generate_keys, AppState};

#[allow(non_snake_case)]
pub fn SignInPage(cx: Scope) -> Element {
    let state = use_shared_state::<AppState>(cx)?;

    let email = use_state(cx, String::new);
    let password = use_state(cx, String::new);

    generate_keys();
    let onsubmit = move |_| {
        let state = state.to_owned();

        let email = email.current().to_string();
        let password = password.current().to_string();

        cx.spawn(async move {
            if let Ok(res) = login(email, password).await {
                let mut state = state.write();
                state.access_toked = Some(res.access_token);
                state.refresh_toked = Some(res.refresh_token);
            }
        })
    };

    cx.render(rsx!(
        form { class: "card-body", onsubmit: onsubmit,
            div { class: "form-control",
                label { class: "label", span { class: "label-text", "Email" } }
                input {
                    class: "input input-bordered",
                    oninput: move |evt| email.set(evt.value.clone())
                }
            }
            div { class: "form-control",
                label { class: "label", span { class: "label-text", "Password" } }
                input {
                    class: "input input-bordered",
                    oninput: move |evt| password.set(evt.value.clone())
                }
            }
            input { class: "btn mt-2 w-full btn-primary", r#type: "submit" }
        }
    ))
}

#[derive(Serialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
}

async fn login(email: String, password: String) -> reqwest::Result<LoginResponse> {
    reqwest::Client::new()
        .post("http://localhost:8080/api/login")
        .header(CONTENT_TYPE, "application/json")
        .json(&LoginRequest { email, password })
        .send()
        .await?
        .json::<LoginResponse>()
        .await
}
