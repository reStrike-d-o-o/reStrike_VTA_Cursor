//! Data import pipelines for bringing external tournament data sets into the
//! application database.
//!
//! Currently this module exposes the Daedo GO2025 importer which reconstructs
//! tournaments from the folder structure produced by the Daedo competition
//! system.

pub mod daedo;
