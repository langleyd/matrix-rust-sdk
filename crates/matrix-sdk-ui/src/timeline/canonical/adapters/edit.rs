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

//! Edit event adapter.
//!
//! Handles m.replace relations (legacy edits only for Epic 1).

use ruma::events::{
    room::message::RoomMessageEventContentWithoutRelation,
    AnySyncMessageLikeEvent, AnySyncTimelineEvent,
};

use super::{message::MessageAdapter, AdapterContext, EventAdapter};
use crate::timeline::canonical::{CanonicalEditState, EditMetadata, MessageContent};

/// Adapter for edit events (m.replace relations).
#[derive(Debug)]
pub(crate) struct EditAdapter;

impl EditAdapter {
    pub(crate) fn new() -> Self {
        EditAdapter
    }

    /// Apply an edit to an existing canonical message.
    fn apply_edit(
        context: &mut AdapterContext<'_>,
        parent_event_id: ruma::OwnedEventId,
        edit_event_id: ruma::OwnedEventId,
        new_content: &RoomMessageEventContentWithoutRelation,
        timestamp: Option<ruma::MilliSecondsSinceUnixEpoch>,
    ) {
        // Get the existing message
        let Some(mut message) = context.state.get_by_event_id(&parent_event_id).cloned() else {
            // Parent doesn't exist yet - buffer this edit
            tracing::debug!(
                "Edit {} arrived before parent {}, buffering",
                edit_event_id,
                parent_event_id
            );
            context.state.add_pending_edit(parent_event_id, edit_event_id);
            return;
        };

        // Extract new content
        let msg_type = MessageAdapter::map_message_type(&new_content.msgtype);
        let body = new_content.msgtype.body().to_owned();
        let formatted = MessageAdapter::extract_formatted(&new_content.msgtype);
        let new_message_content = MessageContent { msg_type, body, formatted };

        // Create or update edit state
        let edit_metadata = EditMetadata {
            edit_id: edit_event_id,
            timestamp,
            position: context.ordering_key,
        };

        if let Some(ref mut edit_state) = message.edit_state {
            // Append to existing edit chain
            edit_state.edit_chain.push(edit_metadata);
            edit_state.current_content = new_message_content;
        } else {
            // First edit - create edit state
            message.edit_state = Some(CanonicalEditState {
                current_content: new_message_content.clone(),
                original_content: message.content.clone(),
                edit_chain: vec![edit_metadata],
            });
            // Update message content to show edited version
            message.content = new_message_content;
        }

        // Update the message in state
        context.state.upsert(message);
    }
}

impl EventAdapter for EditAdapter {
    fn process(&self, event: &AnySyncTimelineEvent, context: &mut AdapterContext<'_>) -> bool {
        // Epic 1 POC: Handle legacy m.replace edits only
        match event {
            AnySyncTimelineEvent::MessageLike(AnySyncMessageLikeEvent::RoomMessage(
                ruma::events::room::message::SyncRoomMessageEvent::Original(message_event),
            )) => {
                // Check if this is an edit (has m.replace relation)
                if let Some(ruma::events::room::message::Relation::Replacement(replacement)) =
                    &message_event.content.relates_to
                {
                    let edit_event_id = message_event.event_id.clone();
                    let parent_event_id = replacement.event_id.clone();
                    let timestamp = Some(message_event.origin_server_ts);

                    // Extract the new content from m.new_content
                    Self::apply_edit(
                        context,
                        parent_event_id,
                        edit_event_id,
                        &replacement.new_content,
                        timestamp,
                    );

                    true
                } else {
                    false // Not an edit
                }
            }
            _ => false,
        }
    }
}
