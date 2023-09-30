//! The domain logic of the game itself, decoupled as much as possible from Bevy
//!
//! We implement the following bevy traits to facilitate the integration with the
//! engine :
//!
//! * Event
//! * Component
//! * Resource

pub mod grid;
pub mod moves;
pub mod tile;
