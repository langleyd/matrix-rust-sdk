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

//! Event adapters for canonical timeline.
//!
//! Adapters translate raw Matrix events into canonical timeline updates.

use ruma::events::AnySyncTimelineEvent;

use super::{state::CanonicalTimelineState, CanonicalOrderingKey};

pub(crate) mod edit;
pub(crate) mod message;

/// Context provided to event adapters.
///
/// Contains state and dependencies needed for event processing.
#[derive(Debug)]
pub(crate) struct AdapterContext<'a> {
    /// Canonical timeline state (for lookups and mutations)
    pub state: &'a mut CanonicalTimelineState,

    /// Ordering key for this event
    pub ordering_key: CanonicalOrderingKey,
}

/// Trait for adapting raw events into canonical timeline updates.
///
/// Each adapter handles a specific category of Matrix events.
pub(crate) trait EventAdapter {
    /// Process an event and update canonical state.
    ///
    /// Returns true if the event was processed, false if ignored.
    fn process(&self, event: &AnySyncTimelineEvent, context: &mut AdapterContext<'_>) -> bool;
}
