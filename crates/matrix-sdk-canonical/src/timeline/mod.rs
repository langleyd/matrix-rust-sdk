//! Timeline state management for canonical representation.

pub mod delta;
pub mod ordering;
pub mod state;

pub use delta::CanonicalDelta;
pub use ordering::CanonicalOrderingKey;
pub use state::CanonicalTimelineState;
