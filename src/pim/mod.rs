//! Azure Privileged Identity Management (PIM) integration.
//!
//! This module provides functionality for:
//! - Fetching eligible PIM roles across Azure subscriptions
//! - Activating roles with justification
//! - Managing active role assignments
//! - Caching and persistence of favorites/settings

// Allow dead code and unused imports in PIM module - full integration pending
#![allow(dead_code)]
#![allow(unused_imports)]

pub mod cache;
pub mod client;
pub mod models;
pub mod settings;

pub use cache::PimCache;
pub use client::PimClient;
pub use models::{
    ActivationRequest, ActiveAssignment, EligibleRole, JustificationPreset, PimApiStatus,
    PimSettings, Subscription,
};
pub use settings::{load_pim_settings, save_pim_settings};
