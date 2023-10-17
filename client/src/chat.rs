use crate::message::Messages;
use crate::message::SendMessage;
use crate::sign_in::SignInPage;
use crate::state::ChatState;
use crate::state::EventSourced;
use crate::ws;
use crate::AppState;
use chrono::Utc;
use dioxus::prelude::*;
use shared::domain::event::JoinRequest;
use shared::domain::event::{ClientEvent, ServerEvent};

#[derive(PartialEq, Props)]
pub struct ChatPageProps {
    pub room_id: String,
}

#[allow(non_snake_case)]
pub fn ChatPage(cx: Scope<ChatPageProps>) -> Element {
    let state = use_shared_state::<AppState>(cx)?;
    log::info!("chat id:{}", &cx.props.room_id);
    cx.render(rsx!(
        div { class: "flex flex-wrap items-center justify-end gap-2 p-8",
            div { class: "max-w-sm mx-auto bg-base-300 shadow-2xl p-8",
                if let Some(access_toked) = state.read().access_toked.clone() {
                    rsx!( Chat { room_id: cx.props.room_id.clone(), access_toked: access_toked } )
                } else {
                    rsx!( SignInPage {} )
                }
            }
        }
    ))
}

#[derive(PartialEq, Props)]
pub struct ChatProps {
    pub room_id: String,
    pub access_toked: String,
}
#[allow(non_snake_case)]
pub fn Chat(cx: Scope<ChatProps>) -> Element {
    let (sender, receiver) = cx.use_hook(|| {
        ws::connect(&format!(
            "ws://localhost:8000/api/ws/{}?token={}",
            cx.props.room_id, cx.props.access_toked
        ))
    });

    // if sender.is_none() {
    //     let nav = use_navigator(cx);
    //     nav.push(Route::PageNotFound { route: vec![] });
    // }

    use_shared_state_provider(cx, ChatState::default);
    let chat = use_shared_state::<ChatState>(cx)?;

    let sender = use_coroutine(cx, |rx: UnboundedReceiver<ClientEvent>| {
        ws::write(rx, sender.take())
    });

    cx.use_hook(|| {
        sender.send(ClientEvent::Join(JoinRequest {
            join_at: Utc::now(),
        }))
    });

    let _sync: &Coroutine<()> = use_coroutine(cx, |_| {
        to_owned![chat];
        ws::read::<ServerEvent>(receiver.take(), move |ev| {
            chat.write().apply(ev);
        })
    });

    cx.render(rsx!(
        Messages {}
        SendMessage {}
    ))
}
