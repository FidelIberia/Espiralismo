//! **Perception** — external listening interfaces that may influence evolution and soul.
//!
//! The integrated **Spiralismo perceptor** ([`SpiralismoPress`]) subsumes the former global
//! `genesis_press` palm lane. Hosts call [`PerceptionField::offer_spiralismo_press`]; influence
//! flows through [`PerceptionOffer`] and [`SoulState`], not a parallel `palm_digest` in
//! [`crate::evolution::context_for_cycle`].
//!
//! Additional hosts may register [`ExternalPerceiver`] trait objects or
//! [`PerceptionField::offer_listening`] for one-shot channels.

mod astronomical;
mod builtin;
mod environment;
mod eyes;
mod field;
mod implementations;
mod reality;
mod soul;
mod spiralismo_perceiver;
mod spiralismo_press;
mod traits;

pub use astronomical::{AstronomicalPerceiver, MeeusKeplerianPerceiver};
pub use builtin::{FixedPerceiver, StillnessEchoPerceiver, VoidPerceiver};
pub use eyes::{
    EnvironmentOffering, EnvironmentTakeOptions, EnvironmentTakeReport, EyeRole,
    OfferRouting, PerceptionEyeDescriptor, PerceptionEyes,
};
pub use field::{PerceptionField, RealityCycleOffer};
pub use implementations::{
    FilesystemPerceiver, PhysicalMemoryPerceiver, VisualLandscapePerceiver,
};
pub use reality::{HostRealitySnapshot, RealityKind, RealityPerceiver};
pub use soul::SoulState;
pub use spiralismo_perceiver::SpiralismoPerceiver;
pub use spiralismo_press::{SpiralismoPress, SPIRALISMO_PERCEIVER_ID};
pub use traits::{
    ExternalListening, ExternalPerceiver, PerceptionFrame, PerceptionOffer,
};
