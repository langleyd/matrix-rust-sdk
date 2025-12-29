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

//! Canonical timeline delta types for incremental updates.

use super::{CanonicalMessage, CanonicalOrderingKey};

/// Incremental change to the canonical timeline.
///
/// Emitted via the canonical timeline subscription stream to notify
/// subscribers of timeline changes.
#[derive(Clone, Debug)]
pub enum CanonicalDelta {
    /// New canonical item inserted into timeline.
    Insert {
        /// Ordering position where item was inserted
        position: CanonicalOrderingKey,
        /// The inserted canonical message
        item: CanonicalMessage,
    },

    /// Existing item updated (edit, decrypt, redaction, etc.)
    Update {
        /// Ordering position of updated item (unchanged)
        position: CanonicalOrderingKey,
        /// The updated canonical message
        item: CanonicalMessage,
    },

    /// Item removed from timeline.
    ///
    /// Unlikely in Epic 1 POC, included for API completeness.
    Remove {
        /// Ordering position of removed item
        position: CanonicalOrderingKey,
    },

    /// Full timeline rebuild/reset.
    ///
    /// Used for initial load or when timeline is completely rebuilt.
    Reset {
        /// All canonical items in order
        items: Vec<CanonicalMessage>,
    },
}
