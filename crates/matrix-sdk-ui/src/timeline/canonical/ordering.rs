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

//! Canonical ordering key for stable timeline item ordering.

use ruma::MilliSecondsSinceUnixEpoch;

/// Stable ordering key for canonical timeline items.
///
/// Epic 1 POC: Uses a simple u64 counter for ordering. Production implementation
/// would integrate with the SDK's LinkedChunk Position type.
///
/// # Stability
///
/// - **STABLE**: Never changes after assignment
/// - **REBUILDABLE**: Can be reconstructed from stored sequence numbers
///
/// # Ordering Guarantees
///
/// - Decryption does NOT change position
/// - Edits do NOT change parent message position
/// - Pagination preserves position ordering
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanonicalOrderingKey(u64);

impl CanonicalOrderingKey {
    /// Create a canonical ordering key from a sequence number.
    ///
    /// Epic 1 POC: Uses a simple counter. Production would use Position.
    pub fn from_sequence(seq: u64) -> Self {
        CanonicalOrderingKey(seq)
    }

    /// Create from timestamp (fallback for Epic 1).
    pub fn from_timestamp(ts: MilliSecondsSinceUnixEpoch) -> Self {
        CanonicalOrderingKey(ts.0.into())
    }

    /// Get the underlying sequence number.
    pub(crate) fn as_u64(&self) -> u64 {
        self.0
    }
}

impl From<u64> for CanonicalOrderingKey {
    fn from(seq: u64) -> Self {
        CanonicalOrderingKey(seq)
    }
}
