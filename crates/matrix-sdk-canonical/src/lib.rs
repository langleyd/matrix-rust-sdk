//! Canonical Timeline Abstraction for Matrix
//!
//! This crate provides a canonical, stable representation of Matrix timeline events
//! that abstracts away protocol-level details and MSC-specific event structures.
//!
//! The canonical timeline separates Matrix specification concerns (handled by SDK engineers)
//! from product semantics (used by application developers), ensuring that Matrix protocol
//! changes don't break client code.

#![warn(missing_docs)]

pub mod adapters;
pub mod timeline;
pub mod types;

// Re-export key types at crate root for convenience
pub use timeline::{CanonicalDelta, CanonicalOrderingKey};
pub use types::{
    CanonicalEditState, CanonicalMessage, ContentAvailability, EditMetadata, FormattedBody,
    MessageContent, MessageType,
};
