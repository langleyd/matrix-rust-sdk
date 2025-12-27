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

//! Canonical timeline state management.
//!
//! Maintains the in-memory canonical timeline state for Epic 1 POC.
//! Uses BTreeMap for sequence-ordered storage.

use std::collections::BTreeMap;

use ruma::OwnedEventId;
use tokio::sync::broadcast;

use super::{CanonicalDelta, CanonicalMessage, CanonicalOrderingKey};

/// In-memory canonical timeline state.
///
/// Epic 1 POC: In-memory only, no persistence.
/// Stores canonical messages ordered by sequence number.
#[derive(Debug)]
pub(crate) struct CanonicalTimelineState {
    /// Canonical messages ordered by sequence
    items: BTreeMap<u64, CanonicalMessage>,

    /// Event ID to sequence lookup for updates
    event_to_sequence: BTreeMap<OwnedEventId, u64>,

    /// Pending edits that arrived before their parent
    /// Maps parent event ID to list of edit event IDs
    pending_edits: BTreeMap<OwnedEventId, Vec<OwnedEventId>>,

    /// Delta broadcast channel for subscribers
    delta_tx: broadcast::Sender<CanonicalDelta>,

    /// Sequence counter for ordering
    next_sequence: u64,
}

impl CanonicalTimelineState {
    /// Create a new empty canonical timeline state.
    pub(crate) fn new() -> Self {
        let (delta_tx, _) = broadcast::channel(128);
        CanonicalTimelineState {
            items: BTreeMap::new(),
            event_to_sequence: BTreeMap::new(),
            pending_edits: BTreeMap::new(),
            delta_tx,
            next_sequence: 0,
        }
    }

    /// Subscribe to canonical timeline deltas.
    pub(crate) fn subscribe(&self) -> broadcast::Receiver<CanonicalDelta> {
        self.delta_tx.subscribe()
    }

    /// Allocate the next sequence number.
    pub(crate) fn next_ordering_key(&mut self) -> CanonicalOrderingKey {
        let seq = self.next_sequence;
        self.next_sequence += 1;
        CanonicalOrderingKey::from_sequence(seq)
    }

    /// Insert or update a canonical message.
    ///
    /// Returns true if this was a new insertion, false if it was an update.
    pub(crate) fn upsert(&mut self, message: CanonicalMessage) -> bool {
        let sequence = message.ordering_key.as_u64();
        let event_id = message.id.clone();

        let is_new = !self.event_to_sequence.contains_key(&event_id);

        self.items.insert(sequence, message.clone());
        self.event_to_sequence.insert(event_id, sequence);

        let delta = if is_new {
            CanonicalDelta::Insert { position: message.ordering_key, item: message }
        } else {
            CanonicalDelta::Update { position: message.ordering_key, item: message }
        };

        let _ = self.delta_tx.send(delta);

        is_new
    }

    /// Get a canonical message by event ID.
    pub(crate) fn get_by_event_id(&self, event_id: &OwnedEventId) -> Option<&CanonicalMessage> {
        let sequence = self.event_to_sequence.get(event_id)?;
        self.items.get(sequence)
    }

    /// Get all canonical messages in order.
    pub(crate) fn items(&self) -> Vec<CanonicalMessage> {
        self.items.values().cloned().collect()
    }

    /// Get the number of items in the timeline.
    #[allow(dead_code)]
    pub(crate) fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if the timeline is empty.
    #[allow(dead_code)]
    pub(crate) fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Register a pending edit that arrived before its parent.
    pub(crate) fn add_pending_edit(&mut self, parent_event_id: OwnedEventId, edit_event_id: OwnedEventId) {
        self.pending_edits.entry(parent_event_id).or_insert_with(Vec::new).push(edit_event_id);
    }

    /// Get and remove pending edits for a parent event.
    pub(crate) fn take_pending_edits(&mut self, parent_event_id: &OwnedEventId) -> Vec<OwnedEventId> {
        self.pending_edits.remove(parent_event_id).unwrap_or_default()
    }

    /// Remove a canonical message by ordering key.
    #[allow(dead_code)]
    pub(crate) fn remove(&mut self, position: CanonicalOrderingKey) -> Option<CanonicalMessage> {
        let seq = position.as_u64();
        let message = self.items.remove(&seq)?;
        self.event_to_sequence.remove(&message.id);

        let delta = CanonicalDelta::Remove { position };
        let _ = self.delta_tx.send(delta);

        Some(message)
    }

    /// Emit a full reset delta with all current items.
    #[allow(dead_code)]
    pub(crate) fn emit_reset(&self) {
        let delta = CanonicalDelta::Reset { items: self.items() };
        let _ = self.delta_tx.send(delta);
    }
}

#[cfg(test)]
mod tests {
    use ruma::{event_id, user_id, MilliSecondsSinceUnixEpoch};
    use super::*;
    use crate::timeline::canonical::{MessageContent, MessageType, ContentAvailability};

    fn create_test_message(event_id: OwnedEventId, body: &str, sequence: u64) -> CanonicalMessage {
        CanonicalMessage {
            id: event_id,
            sender: user_id!("@alice:example.org").to_owned(),
            content: MessageContent {
                msg_type: MessageType::Text,
                body: body.to_string(),
                formatted: None,
            },
            edit_state: None,
            ordering_key: CanonicalOrderingKey::from_sequence(sequence),
            availability: ContentAvailability::Known,
            timestamp: Some(MilliSecondsSinceUnixEpoch::now()),
        }
    }

    #[test]
    fn test_stable_ordering() {
        let mut state = CanonicalTimelineState::new();

        // Insert messages in order
        let msg1 = create_test_message(event_id!("$event1").to_owned(), "First message", 1);
        let msg2 = create_test_message(event_id!("$event2").to_owned(), "Second message", 2);
        let msg3 = create_test_message(event_id!("$event3").to_owned(), "Third message", 3);

        state.upsert(msg1.clone());
        state.upsert(msg2.clone());
        state.upsert(msg3.clone());

        // Verify ordering is stable
        let items = state.items();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].id, msg1.id);
        assert_eq!(items[1].id, msg2.id);
        assert_eq!(items[2].id, msg3.id);
    }

    #[test]
    fn test_message_update_preserves_position() {
        let mut state = CanonicalTimelineState::new();

        let msg = create_test_message(event_id!("$event1").to_owned(), "Original", 1);
        state.upsert(msg.clone());

        // Update the message (e.g., decryption)
        let mut updated_msg = msg.clone();
        updated_msg.content.body = "Decrypted".to_string();
        updated_msg.availability = ContentAvailability::Known;

        state.upsert(updated_msg.clone());

        // Position should be unchanged
        let items = state.items();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].ordering_key, msg.ordering_key);
        assert_eq!(items[0].content.body, "Decrypted");
    }

    #[test]
    fn test_pending_edits() {
        let mut state = CanonicalTimelineState::new();

        let parent_id = event_id!("$parent").to_owned();
        let edit1_id = event_id!("$edit1").to_owned();
        let edit2_id = event_id!("$edit2").to_owned();

        // Add pending edits
        state.add_pending_edit(parent_id.clone(), edit1_id.clone());
        state.add_pending_edit(parent_id.clone(), edit2_id.clone());

        // Retrieve pending edits
        let edits = state.take_pending_edits(&parent_id);
        assert_eq!(edits.len(), 2);
        assert_eq!(edits[0], edit1_id);
        assert_eq!(edits[1], edit2_id);

        // Should be empty after taking
        let edits2 = state.take_pending_edits(&parent_id);
        assert!(edits2.is_empty());
    }

    #[test]
    fn test_delta_broadcast() {
        let mut state = CanonicalTimelineState::new();
        let mut rx = state.subscribe();

        // Insert a message
        let msg = create_test_message(event_id!("$event1").to_owned(), "Test", 1);
        state.upsert(msg.clone());

        // Should receive an Insert delta
        let delta = rx.try_recv().unwrap();
        match delta {
            CanonicalDelta::Insert { position, item } => {
                assert_eq!(position, msg.ordering_key);
                assert_eq!(item.id, msg.id);
            }
            _ => panic!("Expected Insert delta"),
        }
    }

    #[test]
    fn test_event_id_lookup() {
        let mut state = CanonicalTimelineState::new();

        let msg = create_test_message(event_id!("$event1").to_owned(), "Test", 1);
        state.upsert(msg.clone());

        // Should be able to look up by event ID
        let found = state.get_by_event_id(&msg.id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, msg.id);

        // Non-existent event should return None
        let not_found = state.get_by_event_id(&event_id!("$notfound").to_owned());
        assert!(not_found.is_none());
    }
}
