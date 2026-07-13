//! Optional application services for caching, scheduling, events, and realtime transports.

#[cfg(feature = "cache")]
pub mod cache;
#[cfg(feature = "events")]
pub mod events;
#[cfg(feature = "realtime")]
pub mod realtime;
#[cfg(feature = "realtime")]
/// WebSocket gateway runtime: connections, rooms, and broadcasting.
pub mod ws;
#[cfg(feature = "scheduling")]
pub mod scheduling;
