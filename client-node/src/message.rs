use std::fmt::{Debug, Display};

use crate::utils::traits::Serializable;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageType {
    Private,
    Public,
}

pub struct Message {
    pub(crate) id: uuid::Uuid,
    pub type_: MessageType,
    pub author: String,
    pub recipient: Option<String>,
    pub content: String,
}

impl Message {
    pub fn new<S: Into<String>>(
        type_: MessageType,
        author: S,
        recipient: Option<S>,
        content: S,
    ) -> Self {
        Message {
            id: uuid::Uuid::new_v4(),
            type_,
            author: author.into(),
            recipient: recipient.map(|r| r.into()),
            content: content.into(),
        }
    }
}

impl Serializable for Message {
    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend(self.id.as_bytes());
        data.push(match self.type_ {
            MessageType::Private => 0,
            MessageType::Public => 1,
        });
        data.extend(self.author.as_bytes());
        if let Some(ref recipient) = self.recipient {
            data.extend(recipient.as_bytes());
        } else {
            data.push(0); // Null byte for no recipient
        }
        data.extend(self.content.as_bytes());
        data
    }

    fn deserialize(data: &[u8]) -> Self {
        let id = uuid::Uuid::from_slice(&data[0..16]).unwrap();
        let type_ = match data[16] {
            0 => MessageType::Private,
            1 => MessageType::Public,
            _ => panic!("Unknown message type"),
        };
        let author_end = data[17..].iter().position(|&b| b == 0).unwrap() + 17;
        let author = String::from_utf8(data[17..author_end].to_vec()).unwrap();

        let recipient_start = author_end + 1;
        let recipient_end = data[recipient_start..]
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(data.len() - recipient_start)
            + recipient_start;
        let recipient = if recipient_start < recipient_end {
            Some(String::from_utf8(data[recipient_start..recipient_end].to_vec()).unwrap())
        } else {
            None
        };

        let content = String::from_utf8(data[recipient_end + 1..].to_vec()).unwrap();

        Message {
            id,
            type_,
            author,
            recipient,
            content,
        }
    }
}

impl Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Message {{ id: {}, type: {:?}, author: {}, recipient: {:?}, content: {} }}",
            self.id, self.type_, self.author, self.recipient, self.content
        )
    }
}

impl Clone for Message {
    fn clone(&self) -> Self {
        Message {
            id: self.id,
            type_: self.type_.clone(),
            author: self.author.clone(),
            recipient: self.recipient.clone(),
            content: self.content.clone(),
        }
    }
}
