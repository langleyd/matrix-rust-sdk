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

//! Experimental canonical timeline implementation (Epic 1 POC).
//!
//! This module provides a canonical timeline layer that:
//! - Maintains stable ordering for timeline items regardless of decryption or relation arrival
//! - Unifies legacy event edits into a single representation
//! - Exposes availability states (known, encrypted, redacted) without raw event inspection
//!
//! # Epic 1 POC - Implementation Status
//!
//! ## ✅ Implemented
//!
//! - **Stable Ordering (US1)**: Timeline items maintain stable positions using sequence-based
//!   `CanonicalOrderingKey`. Decryption and edits never reorder existing items.
//! - **Edit Handling (US2 partial)**: Legacy m.replace edits are tracked in `CanonicalEditState`
//!   with full edit chain history. Extensible events deferred to Epic 2.
//! - **Availability States (US4)**: Three states tracked via `ContentAvailability`:
//!   - `Known`: Fully decrypted and available
//!   - `Encrypted`: Awaiting decryption (UTD cause tracking in Epic 2)
//!   - `Redacted`: Content removed, edit history cleared
//!
//! ## ❌ Out of Scope (Epic 1)
//!
//! - Thread semantics (US3) → Epic 2
//! - Reaction aggregation (US2 complete) → Epic 2
//! - Timeline rebuild from storage (US5) → Epic 2
//! - Persistent storage (in-memory only for POC) → Epic 2
//! - Integration with TimelineController (placeholder APIs only) → Epic 2
//!
//! # Architecture
//!
//! ## Core Types
//!
//! - [`CanonicalMessage`]: User-visible timeline item with stable identity and ordering
//! - [`CanonicalOrderingKey`]: u64-based sequence number (POC simplification)
//! - [`CanonicalDelta`]: Incremental change (Insert/Update/Remove/Reset)
//! - [`ContentAvailability`]: Known/Encrypted/Redacted state tracking
//!
//! ## Adapters
//!
//! Event processing is delegated to specialized adapters:
//! - [`MessageAdapter`]: Processes m.room.message, m.room.encrypted, redactions
//! - [`EditAdapter`]: Processes m.replace relations (legacy edits)
//!
//! ## State Management
//!
//! [`CanonicalTimelineState`] maintains in-memory timeline with:
//! - BTreeMap storage for ordered items
//! - Broadcast channels for delta subscriptions
//! - Pending edit buffer for out-of-order arrivals
//!
//! # Usage (Experimental API)
//!
//! ```ignore
//! #[cfg(feature = "experimental-canonical-timeline")]
//! {
//!     // Subscribe to canonical timeline
//!     let (initial_items, mut delta_stream) = timeline.subscribe_canonical().await;
//!
//!     // Display initial items
//!     for item in initial_items {
//!         match item.availability {
//!             ContentAvailability::Known => {
//!                 println!("{}: {}", item.sender, item.content.body);
//!             }
//!             ContentAvailability::Encrypted { .. } => {
//!                 println!("{}: [encrypted]", item.sender);
//!             }
//!             ContentAvailability::Redacted => {
//!                 println!("{}: [redacted]", item.sender);
//!             }
//!         }
//!     }
//!
//!     // React to updates
//!     while let Ok(delta) = delta_stream.recv().await {
//!         match delta {
//!             CanonicalDelta::Insert { position, item } => {
//!                 // New message arrived
//!             }
//!             CanonicalDelta::Update { position, item } => {
//!                 // Message decrypted or edited
//!             }
//!             _ => {}
//!         }
//!     }
//! }
//! ```
//!
//! # Limitations (Epic 1 POC)
//!
//! 1. **No Controller Integration**: Timeline API methods return placeholders. Full integration
//!    requires adding CanonicalTimelineState to TimelineController and hooking event processing.
//! 2. **In-Memory Only**: No persistence. Timeline state lost on restart.
//! 3. **Basic Events Only**: m.room.message, m.room.encrypted, m.room.redaction supported.
//!    Reactions, polls, state events ignored.
//! 4. **Simplified Ordering**: Uses u64 sequences instead of LinkedChunk Position for POC.
//! 5. **No UTD Cause Tracking**: Encrypted events marked with `utd_cause: None`.
//!
//! # Future Work (Epic 2+)
//!
//! - Integrate with TimelineController for real event processing
//! - Add persistent storage for canonical projections
//! - Implement thread semantics (US3)
//! - Add reaction aggregation (US2 complete)
//! - Support timeline rebuild from raw events (US5)
//! - Track UTD causes for encrypted events
//! - Integrate with LinkedChunk Position
//! - Add integration tests for all acceptance scenarios

#![cfg(feature = "experimental-canonical-timeline")]

mod adapters;
mod delta;
mod ordering;
mod state;
mod types;

pub use delta::CanonicalDelta;
pub use ordering::CanonicalOrderingKey;
pub use types::{
    CanonicalEditState, CanonicalMessage, ContentAvailability, EditMetadata, FormattedBody,
    MessageContent, MessageType,
};

// Internal exports for timeline integration
pub(crate) use adapters::{edit::EditAdapter, message::MessageAdapter, AdapterContext, EventAdapter};
pub(crate) use state::CanonicalTimelineState;
