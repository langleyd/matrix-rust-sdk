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

//! Canonical timeline data types.

use matrix_sdk_base::crypto::types::events::UtdCause;
use ruma::{MilliSecondsSinceUnixEpoch, OwnedEventId, OwnedUserId};

use super::CanonicalOrderingKey;

/// Content availability state for a canonical timeline item.
///
/// # Transitions
///
/// - `Encrypted` → `Known` (on successful decryption)
/// - Any → `Redacted` (irreversible)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContentAvailability {
    /// Content is fully available and decrypted.
    Known,

    /// Content is encrypted, decryption pending or failed.
    Encrypted {
        /// Reason for decryption failure, if known.
        utd_cause: Option<UtdCause>,
    },

    /// Content has been redacted (removed).
    Redacted,
}

/// Message content representation.
///
/// Abstracts the actual message body and type, hiding Matrix event structure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageContent {
    /// Message type (text, image, file, etc.)
    pub msg_type: MessageType,

    /// Plain text body
    pub body: String,

    /// Formatted body (HTML, markdown, etc.)
    pub formatted: Option<FormattedBody>,
}

impl MessageContent {
    /// Create an empty message content (for encrypted placeholders).
    pub fn empty() -> Self {
        MessageContent {
            msg_type: MessageType::Text,
            body: String::new(),
            formatted: None,
        }
    }

    /// Create a redacted message content.
    pub fn redacted() -> Self {
        MessageContent {
            msg_type: MessageType::Text,
            body: String::from("[redacted]"),
            formatted: None,
        }
    }
}

/// Formatted message body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormattedBody {
    /// Format type (e.g., "org.matrix.custom.html")
    pub format: String,

    /// Formatted content
    pub body: String,
}

/// Message type enumeration.
///
/// Epic 1 POC focuses on text messages. Media types included for completeness
/// but have minimal implementation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MessageType {
    /// Plain text message
    Text,

    /// Image message (minimal POC support)
    Image,

    /// File attachment (minimal POC support)
    File,

    /// Video message (minimal POC support)
    Video,

    /// Audio message (minimal POC support)
    Audio,
}

/// Edit metadata for a single edit event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditMetadata {
    /// Event ID of the edit event
    pub edit_id: OwnedEventId,

    /// Timestamp of the edit
    pub timestamp: Option<MilliSecondsSinceUnixEpoch>,

    /// Ordering position of the edit event
    pub position: CanonicalOrderingKey,
}

/// Edit history state for a canonical message.
///
/// Tracks the edit chain without exposing raw event relations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalEditState {
    /// Current (latest) message content
    pub current_content: MessageContent,

    /// Original (first) message content
    pub original_content: MessageContent,

    /// Edit chain metadata (chronological order)
    pub edit_chain: Vec<EditMetadata>,
}

/// Canonical timeline message item.
///
/// Represents user-visible message content, abstracting away Matrix event structures.
///
/// # Field Stability
///
/// - **STABLE**: `id`, `sender`, `ordering_key` - immutable once set
/// - **OPTIONAL**: `timestamp`, `edit_state` - may be None
/// - **REBUILDABLE**: `content` (via edit resolution), `availability` (via decryption)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalMessage {
    /// Stable unique identifier (from event ID)
    pub id: OwnedEventId,

    /// Sender of the message
    pub sender: OwnedUserId,

    /// Message content (text, HTML, etc.)
    pub content: MessageContent,

    /// Edit history (if message has been edited)
    pub edit_state: Option<CanonicalEditState>,

    /// Stable ordering key (never changes)
    pub ordering_key: CanonicalOrderingKey,

    /// Content availability state
    pub availability: ContentAvailability,

    /// Timestamp from event (optional, unreliable for ordering)
    pub timestamp: Option<MilliSecondsSinceUnixEpoch>,
}
