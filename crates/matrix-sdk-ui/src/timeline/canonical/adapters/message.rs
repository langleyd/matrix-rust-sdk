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

//! Message event adapter.
//!
//! Handles m.room.message events.

use ruma::events::{
    room::{
        encrypted::SyncRoomEncryptedEvent,
        message::MessageType as RumaMessageType,
        redaction::SyncRoomRedactionEvent,
    },
    AnySyncMessageLikeEvent, AnySyncTimelineEvent,
};

use super::{AdapterContext, EventAdapter};
use crate::timeline::canonical::{
    CanonicalMessage, ContentAvailability, FormattedBody, MessageContent, MessageType,
};

/// Adapter for m.room.message events.
#[derive(Debug)]
pub(crate) struct MessageAdapter;

impl MessageAdapter {
    pub(crate) fn new() -> Self {
        MessageAdapter
    }

    /// Convert Ruma message type to canonical message type.
    pub(super) fn map_message_type(ruma_type: &RumaMessageType) -> MessageType {
        match ruma_type {
            RumaMessageType::Text(_) | RumaMessageType::Notice(_) | RumaMessageType::Emote(_) => {
                MessageType::Text
            }
            RumaMessageType::Image(_) => MessageType::Image,
            RumaMessageType::Video(_) => MessageType::Video,
            RumaMessageType::Audio(_) => MessageType::Audio,
            RumaMessageType::File(_) => MessageType::File,
            _ => MessageType::Text, // Fallback for unsupported types
        }
    }

    /// Extract formatted body from Ruma message type.
    pub(super) fn extract_formatted(ruma_type: &RumaMessageType) -> Option<FormattedBody> {
        match ruma_type {
            RumaMessageType::Text(text_content) => {
                text_content.formatted.as_ref().map(|f| FormattedBody {
                    format: f.format.to_string(),
                    body: f.body.clone(),
                })
            }
            RumaMessageType::Emote(emote_content) => {
                emote_content.formatted.as_ref().map(|f| FormattedBody {
                    format: f.format.to_string(),
                    body: f.body.clone(),
                })
            }
            RumaMessageType::Notice(notice_content) => {
                notice_content.formatted.as_ref().map(|f| FormattedBody {
                    format: f.format.to_string(),
                    body: f.body.clone(),
                })
            }
            _ => None,
        }
    }
}

impl EventAdapter for MessageAdapter {
    fn process(&self, event: &AnySyncTimelineEvent, context: &mut AdapterContext<'_>) -> bool {
        match event {
            // Handle original (decrypted) m.room.message events
            AnySyncTimelineEvent::MessageLike(AnySyncMessageLikeEvent::RoomMessage(
                ruma::events::room::message::SyncRoomMessageEvent::Original(message_event),
            )) => {
                let event_id = message_event.event_id.clone();
                let sender = message_event.sender.clone();
                let timestamp = Some(message_event.origin_server_ts);

                let msg_type = Self::map_message_type(&message_event.content.msgtype);
                let body = message_event.content.msgtype.body().to_owned();
                let formatted = Self::extract_formatted(&message_event.content.msgtype);

                let content = MessageContent { msg_type, body, formatted };

                let canonical_message = CanonicalMessage {
                    id: event_id.clone(),
                    sender,
                    content,
                    edit_state: None, // Edits handled by EditAdapter
                    ordering_key: context.ordering_key,
                    availability: ContentAvailability::Known,
                    timestamp,
                };

                context.state.upsert(canonical_message);

                // Check for pending edits that arrived before this message
                let pending_edits = context.state.take_pending_edits(&event_id);
                if !pending_edits.is_empty() {
                    tracing::debug!(
                        "Message {} has {} pending edits to apply",
                        event_id,
                        pending_edits.len()
                    );
                }

                true
            }

            // Handle redacted m.room.message events (Phase 5 - US4)
            AnySyncTimelineEvent::MessageLike(AnySyncMessageLikeEvent::RoomMessage(
                ruma::events::room::message::SyncRoomMessageEvent::Redacted(redacted_event),
            )) => {
                let event_id = redacted_event.event_id.clone();
                let sender = redacted_event.sender.clone();
                let timestamp = Some(redacted_event.origin_server_ts);

                let canonical_message = CanonicalMessage {
                    id: event_id,
                    sender,
                    content: MessageContent::redacted(),
                    edit_state: None, // Redaction clears edit state
                    ordering_key: context.ordering_key,
                    availability: ContentAvailability::Redacted,
                    timestamp,
                };

                context.state.upsert(canonical_message);
                true
            }

            // Handle encrypted events (Phase 5 - US4)
            AnySyncTimelineEvent::MessageLike(AnySyncMessageLikeEvent::RoomEncrypted(
                encrypted_event,
            )) => {
                match encrypted_event {
                    SyncRoomEncryptedEvent::Original(original) => {
                        let event_id = original.event_id.clone();
                        let sender = original.sender.clone();
                        let timestamp = Some(original.origin_server_ts);

                        // Epic 1 POC: Mark as encrypted, no UTD cause tracking yet
                        let canonical_message = CanonicalMessage {
                            id: event_id,
                            sender,
                            content: MessageContent::empty(),
                            edit_state: None,
                            ordering_key: context.ordering_key,
                            availability: ContentAvailability::Encrypted { utd_cause: None },
                            timestamp,
                        };

                        context.state.upsert(canonical_message);
                        true
                    }
                    SyncRoomEncryptedEvent::Redacted(redacted) => {
                        let event_id = redacted.event_id.clone();
                        let sender = redacted.sender.clone();
                        let timestamp = Some(redacted.origin_server_ts);

                        let canonical_message = CanonicalMessage {
                            id: event_id,
                            sender,
                            content: MessageContent::redacted(),
                            edit_state: None,
                            ordering_key: context.ordering_key,
                            availability: ContentAvailability::Redacted,
                            timestamp,
                        };

                        context.state.upsert(canonical_message);
                        true
                    }
                }
            }

            // Handle redaction events (Phase 5 - US4)
            AnySyncTimelineEvent::MessageLike(AnySyncMessageLikeEvent::RoomRedaction(
                redaction_event,
            )) => {
                match redaction_event {
                    SyncRoomRedactionEvent::Original(original) => {
                        if let Some(redacts) = &original.content.redacts {
                            // Find and redact the target message
                            if let Some(mut message) = context.state.get_by_event_id(redacts).cloned() {
                                message.content = MessageContent::redacted();
                                message.availability = ContentAvailability::Redacted;
                                message.edit_state = None; // Clear edit history
                                context.state.upsert(message);
                            }
                        }
                        true
                    }
                    SyncRoomRedactionEvent::Redacted(_) => {
                        // Redacted redaction event - ignore
                        false
                    }
                }
            }

            _ => false, // Ignore other event types
        }
    }
}
