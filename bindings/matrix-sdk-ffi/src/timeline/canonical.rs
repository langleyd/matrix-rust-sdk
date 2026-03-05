// Copyright 2025 The Matrix.org Foundation C.I.C.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! FFI bindings for canonical timeline API (Epic 1 POC)

use std::sync::Arc;

use matrix_sdk_ui::timeline::canonical;

/// Content availability state for a canonical timeline item.
#[derive(uniffi::Enum, Clone, Debug, PartialEq, Eq)]
pub enum ContentAvailability {
    /// Content is fully available and decrypted.
    Known,

    /// Content is encrypted, decryption pending or failed.
    Encrypted {
        /// Reason for decryption failure, if known.
        utd_cause: Option<String>,
    },

    /// Content has been redacted (removed).
    Redacted,
}

impl From<canonical::ContentAvailability> for ContentAvailability {
    fn from(value: canonical::ContentAvailability) -> Self {
        match value {
            canonical::ContentAvailability::Known => ContentAvailability::Known,
            canonical::ContentAvailability::Encrypted { utd_cause } => {
                ContentAvailability::Encrypted {
                    utd_cause: utd_cause.map(|cause| format!("{:?}", cause)),
                }
            }
            canonical::ContentAvailability::Redacted => ContentAvailability::Redacted,
        }
    }
}

/// Message type enumeration.
#[derive(uniffi::Enum, Clone, Debug, PartialEq, Eq)]
pub enum CanonicalMessageType {
    /// Plain text message
    Text,

    /// Image message
    Image,

    /// File attachment
    File,

    /// Video message
    Video,

    /// Audio message
    Audio,
}

impl From<canonical::MessageType> for CanonicalMessageType {
    fn from(value: canonical::MessageType) -> Self {
        match value {
            canonical::MessageType::Text => CanonicalMessageType::Text,
            canonical::MessageType::Image => CanonicalMessageType::Image,
            canonical::MessageType::File => CanonicalMessageType::File,
            canonical::MessageType::Video => CanonicalMessageType::Video,
            canonical::MessageType::Audio => CanonicalMessageType::Audio,
        }
    }
}

/// Formatted message body.
#[derive(uniffi::Record, Clone, Debug, PartialEq, Eq)]
pub struct CanonicalFormattedBody {
    /// Format type (e.g., "org.matrix.custom.html")
    pub format: String,

    /// Formatted content
    pub body: String,
}

impl From<canonical::FormattedBody> for CanonicalFormattedBody {
    fn from(value: canonical::FormattedBody) -> Self {
        CanonicalFormattedBody {
            format: value.format,
            body: value.body,
        }
    }
}

/// Message content representation.
#[derive(uniffi::Record, Clone, Debug, PartialEq, Eq)]
pub struct CanonicalMessageContent {
    /// Message type (text, image, file, etc.)
    pub msg_type: CanonicalMessageType,

    /// Plain text body
    pub body: String,

    /// Formatted body (HTML, markdown, etc.)
    pub formatted: Option<CanonicalFormattedBody>,
}

impl From<canonical::MessageContent> for CanonicalMessageContent {
    fn from(value: canonical::MessageContent) -> Self {
        CanonicalMessageContent {
            msg_type: value.msg_type.into(),
            body: value.body,
            formatted: value.formatted.map(Into::into),
        }
    }
}

/// Edit metadata for a single edit event.
#[derive(uniffi::Record, Clone, Debug, PartialEq, Eq)]
pub struct EditMetadata {
    /// Event ID of the edit event
    pub edit_id: String,

    /// Timestamp of the edit (milliseconds since Unix epoch)
    pub timestamp: Option<u64>,

    /// Ordering position of the edit event
    pub position: u64,
}

impl From<canonical::EditMetadata> for EditMetadata {
    fn from(value: canonical::EditMetadata) -> Self {
        EditMetadata {
            edit_id: value.edit_id.to_string(),
            timestamp: value.timestamp.map(|ts| ts.get().into()),
            position: value.position.as_u64(),
        }
    }
}

/// Edit history state for a canonical message.
#[derive(uniffi::Record, Clone, Debug, PartialEq, Eq)]
pub struct CanonicalEditState {
    /// Current (latest) message content
    pub current_content: CanonicalMessageContent,

    /// Original (first) message content
    pub original_content: CanonicalMessageContent,

    /// Edit chain metadata (chronological order)
    pub edit_chain: Vec<EditMetadata>,
}

impl From<canonical::CanonicalEditState> for CanonicalEditState {
    fn from(value: canonical::CanonicalEditState) -> Self {
        CanonicalEditState {
            current_content: value.current_content.into(),
            original_content: value.original_content.into(),
            edit_chain: value.edit_chain.into_iter().map(Into::into).collect(),
        }
    }
}

/// Canonical timeline message item.
///
/// Represents user-visible message content, abstracting away Matrix event structures.
#[derive(uniffi::Record, Clone, Debug, PartialEq, Eq)]
pub struct CanonicalMessage {
    /// Stable unique identifier (event ID)
    pub id: String,

    /// Sender of the message
    pub sender: String,

    /// Sender's display name (if available)
    pub sender_display_name: Option<String>,

    /// Sender's avatar URL (if available)
    pub sender_avatar_url: Option<String>,

    /// Message content (text, HTML, etc.)
    pub content: CanonicalMessageContent,

    /// Edit history (if message has been edited)
    pub edit_state: Option<CanonicalEditState>,

    /// Stable ordering key (never changes)
    pub ordering_key: u64,

    /// Content availability state
    pub availability: ContentAvailability,

    /// Timestamp from event (milliseconds since Unix epoch, optional)
    pub timestamp: Option<u64>,
}

impl From<canonical::CanonicalMessage> for CanonicalMessage {
    fn from(value: canonical::CanonicalMessage) -> Self {
        CanonicalMessage {
            id: value.id.to_string(),
            sender: value.sender.to_string(),
            sender_display_name: value.sender_display_name,
            sender_avatar_url: value.sender_avatar_url,
            content: value.content.into(),
            edit_state: value.edit_state.map(Into::into),
            ordering_key: value.ordering_key.as_u64(),
            availability: value.availability.into(),
            timestamp: value.timestamp.map(|ts| ts.get().into()),
        }
    }
}

/// Incremental change to the canonical timeline.
#[derive(uniffi::Enum, Clone, Debug)]
pub enum CanonicalDelta {
    /// New canonical item inserted into timeline.
    Insert {
        /// Ordering position where item was inserted
        position: u64,
        /// The inserted canonical message
        item: CanonicalMessage,
    },

    /// Existing item updated (edit, decrypt, redaction, etc.)
    Update {
        /// Ordering position of updated item (unchanged)
        position: u64,
        /// The updated canonical message
        item: CanonicalMessage,
    },

    /// Item removed from timeline.
    Remove {
        /// Ordering position of removed item
        position: u64,
    },

    /// Full timeline rebuild/reset.
    Reset {
        /// All canonical items in order
        items: Vec<CanonicalMessage>,
    },
}

impl From<canonical::CanonicalDelta> for CanonicalDelta {
    fn from(value: canonical::CanonicalDelta) -> Self {
        match value {
            canonical::CanonicalDelta::Insert { position, item } => {
                CanonicalDelta::Insert {
                    position: position.as_u64(),
                    item: item.into(),
                }
            }
            canonical::CanonicalDelta::Update { position, item } => {
                CanonicalDelta::Update {
                    position: position.as_u64(),
                    item: item.into(),
                }
            }
            canonical::CanonicalDelta::Remove { position } => {
                CanonicalDelta::Remove {
                    position: position.as_u64(),
                }
            }
            canonical::CanonicalDelta::Reset { items } => {
                CanonicalDelta::Reset {
                    items: items.into_iter().map(Into::into).collect(),
                }
            }
        }
    }
}

/// Callback interface for canonical timeline updates.
#[uniffi::export(callback_interface)]
pub trait CanonicalTimelineListener: Send + Sync {
    /// Called when a delta update is available.
    fn on_update(&self, delta: CanonicalDelta);
}

/// Handle for canonical timeline listener subscription.
#[derive(uniffi::Object)]
pub struct CanonicalTimelineListenerHandle {
    abort_handle: tokio::sync::Mutex<Option<matrix_sdk_common::executor::AbortHandle>>,
}

impl CanonicalTimelineListenerHandle {
    pub(crate) fn new(abort_handle: matrix_sdk_common::executor::AbortHandle) -> Arc<Self> {
        Arc::new(Self {
            abort_handle: tokio::sync::Mutex::new(Some(abort_handle)),
        })
    }
}

#[uniffi::export]
impl CanonicalTimelineListenerHandle {
    /// Cancel the subscription.
    pub async fn cancel(&self) {
        if let Some(handle) = self.abort_handle.lock().await.take() {
            handle.abort();
        }
    }
}
