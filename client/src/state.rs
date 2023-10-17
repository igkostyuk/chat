use crate::message::MessageProps;
use shared::domain::event::{JoinResponse, ServerEvent, UserJoinResponse};
use shared::domain::{Message, MessageContent, User, UserId};
use std::collections::HashMap;

pub trait EventSourced<Ev: ?Sized> {
    fn apply(&mut self, event: Ev);
}

#[derive(Default)]
pub struct ChatState {
    pub user_id: UserId,
    pub users: HashMap<UserId, User>,
    pub messages: Vec<Message>,
}

impl ChatState {
    pub fn get_message_props(&self) -> Vec<MessageProps> {
        self.messages
            .iter()
            .map(|m| MessageProps {
                id: m.id,
                user_name: self.get_user_name(&m.user_id),
                content: self.decode(&m.content),
                created_at: m.created_at,
                is_my: m.user_id.eq(&self.user_id),
            })
            .collect()
    }

    fn get_user_name(&self, user_id: &UserId) -> String {
        self.users
            .get(user_id)
            .map(|u| u.name.as_ref())
            .unwrap_or("unknown")
            .to_string()
    }

    fn decode(&self, content: &MessageContent) -> String {
        content.as_ref().to_string()
    }
}

impl EventSourced<Message> for ChatState {
    fn apply(&mut self, ev: Message) {
        self.messages.push(ev)
    }
}

impl EventSourced<JoinResponse> for ChatState {
    fn apply(&mut self, ev: JoinResponse) {
        self.user_id = ev.user_id
    }
}

impl EventSourced<UserJoinResponse> for ChatState {
    fn apply(&mut self, ev: UserJoinResponse) {}
}

impl EventSourced<ServerEvent> for ChatState {
    fn apply(&mut self, ev: ServerEvent) {
        match ev {
            ServerEvent::ErrMessage(_) => todo!(),
            ServerEvent::Join(ev) => self.apply(ev),
            ServerEvent::ReceivedMessage(ev) => self.apply(ev),
            ServerEvent::UserJoin(ev) => self.apply(ev),
        }
    }
}
