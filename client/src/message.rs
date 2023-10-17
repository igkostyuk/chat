use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use shared::domain::{
    event::{ClientEvent, JoinRequest},
    MessageContent, MessageId,
};

use crate::state::ChatState;

#[allow(non_snake_case)]
pub fn Messages(cx: Scope) -> Element {
    let chat = use_shared_state::<ChatState>(cx)?;
    cx.render(rsx!(
        div { class: "overflow:scroll overflow-y-auto w-full",
            chat.read().get_message_props().into_iter().map(|mp| rsx!(Message {
            id: mp.id,
            user_name: mp.user_name,
            content: mp.content,
            created_at: mp.created_at,
            is_my: mp.is_my,
        }))
        }
    ))
}

#[derive(PartialEq, Props)]
pub struct MessageProps {
    pub id: MessageId,
    pub user_name: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub is_my: bool,
}
#[allow(non_snake_case)]
fn Message(cx: Scope<MessageProps>) -> Element {
    cx.render(rsx!(
        div { key: "{cx.props.id.as_ref()}", class: if cx.props.is_my { "chat chat-start " } else { "chat chat-end" },
            div { class: "chat-header",
                "{cx.props.user_name}"
                time { class: "text-xs opacity-50",
                    format!("{}", cx.props.created_at.format("%d/%m/%Y %H:%M"))
                }
            }
            div {
                onmounted: move |cx| {
                    cx.inner().clone().scroll_to(ScrollBehavior::Smooth);
                },
                class: "chat-bubble",
                "{cx.props.content}"
            }
        }
    ))
}

#[allow(non_snake_case)]
pub fn SendMessage(cx: Scope) -> Element {
    let sender = use_coroutine_handle::<ClientEvent>(cx)?;
    let content = use_state(cx, || "".to_string());
    let color = use_state(cx, || "");

    let onsubmit = move |_| match MessageContent::try_from(content.to_string()) {
        Ok(msg) => {
            sender.send(ClientEvent::SendMessage(msg));
            content.set("".to_string());
            color.set("");
        }
        Err(_) => color.set("input-error"),
    };

    cx.render(rsx!(
        form { class: "w-full", onsubmit: onsubmit,
            input {
                placeholder: "Type here",
                value: "{content}",
                oninput: move |evt| content.set(evt.value.clone()),
                class: "input input-bordered {color} w-full max-w-xs"
            }
        }
    ))
}
