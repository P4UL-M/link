use std::fmt::Debug;

use crate::utils::traits::Serializable;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageType {
    Private,
    Public,
}

pub struct Message {
    pub id: uuid::Uuid,
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

    fn deserialize(data: &[u8]) -> Result<Self, String> {
        if data.len() < 17 {
            return Err("Data too short to contain UUID and message type".into());
        }

        let id =
            uuid::Uuid::from_slice(&data[0..16]).map_err(|_| "Failed to parse UUID from data")?;

        let type_ = match data[16] {
            0 => MessageType::Private,
            1 => MessageType::Public,
            _ => return Err("Unknown message type".into()),
        };

        let author_end = data[17..]
            .iter()
            .position(|&b| b == 0)
            .map(|pos| pos + 17)
            .ok_or("Missing null terminator after author")?;

        let author = String::from_utf8(data[17..author_end].to_vec())
            .map_err(|e| format!("Invalid UTF-8 in author: {}", e))?;

        let recipient_start = author_end + 1;
        if recipient_start >= data.len() {
            return Err("Missing content after author".into());
        }

        let recipient_end = data[recipient_start..]
            .iter()
            .position(|&b| b == 0)
            .map(|pos| pos + recipient_start)
            .unwrap_or(data.len());

        let recipient = if recipient_start < recipient_end {
            Some(
                String::from_utf8(data[recipient_start..recipient_end].to_vec())
                    .map_err(|e| format!("Invalid UTF-8 in recipient: {}", e))?,
            )
        } else {
            None
        };

        let content_start = if recipient_end < data.len() {
            recipient_end + 1
        } else {
            data.len()
        };

        let content = String::from_utf8(data[content_start..].to_vec())
            .map_err(|e| format!("Invalid UTF-8 in content: {}", e))?;

        Ok(Message {
            id,
            type_,
            author,
            recipient,
            content,
        })
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
