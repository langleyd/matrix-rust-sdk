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

//! Bridge module for canonical timeline integration.
//!
//! Re-exports types from matrix-sdk-canonical for internal use within matrix-sdk-ui.
//! This provides a stable internal API boundary.

#[cfg(feature = "experimental-canonical-timeline")]
pub(crate) use matrix_sdk_canonical::{
    CanonicalDelta, CanonicalEditState, CanonicalMessage, CanonicalOrderingKey,
    ContentAvailability, EditMetadata, MessageContent, MessageType,
};

#[cfg(feature = "experimental-canonical-timeline")]
pub(crate) use matrix_sdk_canonical::timeline::state::CanonicalTimelineState;

#[cfg(feature = "experimental-canonical-timeline")]
pub(crate) use matrix_sdk_canonical::adapters::{
    edit::EditAdapter, message::MessageAdapter, AdapterContext, EventAdapter, Profile,
};
