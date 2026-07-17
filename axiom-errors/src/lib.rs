//! # axiom-errors
//!
//! Structured error responses for AXIOM, implementing
//! `docs/AXIOM_YellowPaper_Errors.md`.
//!
//! This crate is the canonical source of:
//!
//! - `ErrorResponse` — the wire format struct returned by Core, Lambda,
//!   Nabla, and ANTIE when any operation fails.
//! - `ErrorCategory` — the 5-way client dispatch category.
//! - `ErrorCode` — stable string codes that clients match on.
//! - `RecoveryHint` — the 9-variant recovery dispatch enum.
//! - `ErrorDetail` — typed detail variants with CBOR tag allocations.
//! - `RequestEnvelope` — wallet-owner-authenticated request envelope
//!   used for debug disclosure (admin and query endpoints).
//! - `CBOR_TAGS` — AXIOM's private-use CBOR tag registry.
//!
//! # Stability
//!
//! **Error codes, CBOR tags, and category discriminants are stable
//! across releases.** Once an entry appears in a tagged release, it
//! MUST NOT change. New entries may be added; old entries may be
//! deprecated but not repurposed.
//!
//! # Compatibility
//!
//! This crate is `no_std` compatible (turn off the `std` feature) for
//! use in the AVM guest ELF. Host-side callers (Lambda, Nabla, ANTIE,
//! client SDK) use the default `std` feature.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod cbor_tags;
pub mod error_category;
pub mod error_code;
pub mod error_detail;
pub mod error_response;
pub mod recovery_hint;
pub mod request_envelope;

// Re-export the main public types at the crate root for convenience.
pub use error_category::ErrorCategory;
pub use error_code::ErrorCode;
pub use error_detail::{
    BalanceDetail, BundleFieldMismatch, ChequeBundleDetail, ErrorDetail, LockReason,
    RateLimitDetail, SabrInsufficientOverlapDetail, StateChainMismatchDetail, VbcLifecycleDetail,
    WalletLockDetail,
};
pub use error_response::ErrorResponse;
pub use recovery_hint::RecoveryHint;
pub use request_envelope::{RequestEnvelope, RequestType};

/// Wire format version constant. Bump this only when the `ErrorResponse`
/// schema changes in a way clients must distinguish.
pub const ERROR_RESPONSE_VERSION: u8 = 1;
