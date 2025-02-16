use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Scope {
    Private,
    Lone {
        lone_id: u32,
    },
    Room {
        lone_id: u32,
        room_id: u32,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Author {
    System,
    User { id: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "format", rename_all = "camelCase")]
pub enum ChatText {
    PlainText { body: String },
    Markdown { body: String },
    Html { body: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "mime")]
pub enum MediaMeta {
    #[serde(rename = "image/jpeg")]
    ImageJPEG { size: u32, dimensions: (u32, u32) },
    #[serde(rename = "image/png")]
    ImagePNG { size: u32, dimensions: (u32, u32) },
    Html { size: u32, dimensions: (u32, u32) },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ChatContent {
    Text(ChatText),
    Image {
        file_id:        u32,
        thumbnail_id:   u32,
        meta: MediaMeta,
    },
    File {
        file_id:    u32,
        filename:   String,
        meta: MediaMeta,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Event {
    System {
        event_id:   u32,
        message:    String,
    },
    Chat {
        event_id:   u32,
        content: ChatContent,
        quote:   Option<u32>,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payload {
    pub(crate) id:         u32,
    pub(crate) author:     Author,
    pub(crate) scope:      Scope,
    pub(crate) event:      Event,
}

impl Payload {
    pub fn new(id: u32, author: Author, scope: Scope, event: Event) -> Self {
        Payload {
            id,
            author,
            scope,
            event,
        }
    }
}