use dioxus::prelude::*;

use reqwest::header::CONTENT_TYPE;
use serde::{Deserialize, Serialize};

use crate::{encryption::generate_keys, AppState};

#[allow(non_snake_case)]
pub fn KeysPage(cx: Scope) -> Element {
    let signing_key = use_state(cx, String::new);
    let verifying_key = use_state(cx, String::new);

    let generate = move |_| {
        let (sign, verify) = generate_keys();
        signing_key.set(sign);
        verifying_key.set(verify);
    };

    let onsubmit = move |_| {
        // let state = state.to_owned();

        // let email = signing_key.current().to_string();
        // let password = verifying_key.current().to_string();

        // cx.spawn(async move {
        //     if let Ok(res) = login(email, password).await {
        //         let mut state = state.write();
        //         state.access_toked = Some(res.access_token);
        //         state.refresh_toked = Some(res.refresh_token);
        //     }
        // })
    };

    cx.render(rsx!(
        div { class: "bg-gray-100 h-screen flex items-center justify-center",
            div { class: "card flex-shrink-0 w-full max-w-sm shadow-2xl bg-base-100",
                form { class: "card-body", onsubmit: onsubmit,
                    div { class: "form-control",
                        label { class: "label", span { class: "label-text", "Signing key" } }
                        input {
                            class: "input input-bordered",
                            value: "{signing_key}",
                            oninput: move |evt| signing_key.set(evt.value.clone())
                        }
                    }
                    div { class: "form-control",
                        label { class: "label", span { class: "label-text", "Verifying key" } }
                        input {
                            class: "input input-bordered",
                            // disabled: true,
                            value: "{verifying_key}",
                            oninput: move |evt| verifying_key.set(evt.value.clone())
                        }
                    }
                    button { class: "btn mt-2 w-full btn-primary", onclick: generate, "generate" }
                }
            }
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
