//! Request authentication envelope for debug disclosure. See
//! `AXIOM_YellowPaper_Errors.md` §12.
//!
//! Used to wrap non-transaction admin and query requests with a
//! wallet-owner or operator signature, so the server can determine
//! when to disclose debug-gated fields in `ErrorResponse.detail`.
//!
//! # Not for transactions
//!
//! Witness and redeem requests do NOT use `RequestEnvelope`. They're
//! already self-authenticating via `client_pk` + `client_sig` end-to-end
//! through Core's CL2/CL3 validation. The envelope is purely for
//! admin/query paths that need debug disclosure.

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

/// Wraps a request payload `T` with signature, nonce, and timestamp
/// so the server can authenticate the requester as the wallet owner
/// (for debug-gated field disclosure) or as an operator (for admin
/// commands).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequestEnvelope<T> {
    /// The Ed25519 public key of the actor claiming authorization.
    /// For wallet-owner requests, must equal the wallet being
    /// queried. For operator requests, must be in the operator
    /// keyring.
    pub actor_pk: [u8; 32],

    /// Type of request being wrapped. Part of the signing message
    /// to prevent cross-purpose signature reuse — a signature for
    /// `QueryStoredState` cannot be replayed as `ExportWalletSeed`.
    pub request_type: RequestType,

    /// Strictly monotonic nonce per (actor_pk, request_type),
    /// persisted server-side. Client MUST bump this on every request
    /// and never decrement. Primary anti-replay defense.
    pub nonce: u64,

    /// Client's view of wall clock time, seconds since Unix epoch.
    /// Server rejects if `|server_now - timestamp_unix| > 60s`.
    /// Secondary anti-replay defense (backup for nonce).
    pub timestamp_unix: u64,

    /// The actual request body. Serialized as CBOR when computing
    /// the signing message.
    pub payload: T,

    /// Ed25519 signature over the signing message defined in
    /// `compute_signing_message`. Wire format is `Vec<u8>` to match
    /// the rest of the AXIOM codebase; MUST be exactly 64 bytes.
    /// Servers verify the length at envelope deserialization time.
    pub signature: Vec<u8>,
}

/// The type of request a `RequestEnvelope` wraps. Appears in the
/// signing message to prevent cross-purpose signature replay.
///
/// New variants append to the end. Discriminants are `#[repr(u16)]`
/// and are stable across releases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(u16)]
pub enum RequestType {
    /// GET wallet's stored state (debug-disclosure gate).
    QueryStoredState = 0,

    /// GET wallet's transaction history.
    QueryHistory = 1,

    /// GET wallet's balance with structured detail.
    QueryBalance = 2,

    /// Admin: export the encrypted wallet seed. Dangerous; logged
    /// prominently. Requires wallet-owner authentication.
    ExportWalletSeed = 100,

    /// Operator: halt the validator. Requires operator keyring.
    AdminHalt = 1000,

    /// Operator: update config at runtime. Requires operator keyring.
    AdminConfigUpdate = 1001,

    /// Operator: force a manual Nabla re-registration of a specific
    /// wallet (recovery tool).
    AdminForceRegister = 1002,
    // Extend as new admin operations land.
}

/// Domain-tag string for the signing message. Included to prevent
/// the signature from being valid in any other protocol context.
pub const REQUEST_ENVELOPE_DOMAIN_TAG: &[u8] = b"AXIOM_REQUEST_ENVELOPE_V1";

/// Compute the signing message for a `RequestEnvelope`. Caller is
/// responsible for having already serialized the payload to CBOR
/// and hashed it with BLAKE3 into `payload_hash`.
///
/// ```text
/// sig_msg = BLAKE3(
///     "AXIOM_REQUEST_ENVELOPE_V1"
///     || actor_pk                    (32 bytes)
///     || (request_type as u16 LE)    (2 bytes)
///     || (nonce LE)                  (8 bytes)
///     || (timestamp_unix LE)         (8 bytes)
///     || payload_hash                (32 bytes)
/// )
/// ```
///
/// Returns the 32-byte BLAKE3 digest. The caller signs this with
/// Ed25519.
///
/// This function is in the `axiom-errors` crate so both servers and
/// clients compute the same bytes — no "one side has a bug" risk.
#[cfg(feature = "std")]
pub fn compute_signing_message(
    actor_pk: &[u8; 32],
    request_type: RequestType,
    nonce: u64,
    timestamp_unix: u64,
    payload_hash: &[u8; 32],
) -> [u8; 32] {
    use blake3::Hasher;
    let mut h = Hasher::new();
    h.update(REQUEST_ENVELOPE_DOMAIN_TAG);
    h.update(actor_pk);
    h.update(&(request_type as u16).to_le_bytes());
    h.update(&nonce.to_le_bytes());
    h.update(&timestamp_unix.to_le_bytes());
    h.update(payload_hash);
    *h.finalize().as_bytes()
}

/// Trivial helper: hash a payload struct with CBOR + BLAKE3.
/// Returns `None` only if CBOR serialization fails, which should be
/// impossible for any well-formed `T: Serialize`.
#[cfg(feature = "std")]
pub fn hash_payload_cbor<T: Serialize>(payload: &T) -> Option<[u8; 32]> {
    let mut buf = alloc::vec::Vec::new();
    ciborium::into_writer(payload, &mut buf).ok()?;
    Some(*blake3::hash(&buf).as_bytes())
}

/// Acceptable clock skew in seconds. Server rejects envelopes whose
/// `timestamp_unix` is more than this many seconds away from the
/// server's current time (in either direction).
pub const MAX_CLOCK_SKEW_SECS: u64 = 60;

/// Errors produced while verifying a `RequestEnvelope`. These map
/// 1:1 to the `E_REQUEST_ENVELOPE_*` error codes and are intended to
/// be converted to `ErrorResponse` at the server's request entry point.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvelopeVerifyError {
    /// The envelope failed to deserialize.
    Malformed(String),
    /// `|server_now - timestamp_unix| > MAX_CLOCK_SKEW_SECS`.
    ClockSkew { server_now: u64, client_ts: u64 },
    /// `nonce <= last_seen_nonce` for this `(actor_pk, request_type)`.
    Replay {
        last_seen_nonce: u64,
        provided_nonce: u64,
    },
    /// Ed25519 signature verification failed.
    InvalidSignature,
    /// `actor_pk` is not authorized for this `request_type` (e.g.
    /// wallet-owner on an operator request).
    Unauthorized,
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct ExamplePayload {
        kind: String,
        value: u64,
    }

    #[test]
    fn envelope_cbor_roundtrip() {
        let env = RequestEnvelope {
            actor_pk: [0xaa; 32],
            request_type: RequestType::QueryStoredState,
            nonce: 42,
            timestamp_unix: 1776000000,
            payload: ExamplePayload {
                kind: String::from("state"),
                value: 1000,
            },
            signature: vec![0xbb; 64],
        };
        let mut buf = Vec::new();
        ciborium::into_writer(&env, &mut buf).unwrap();
        let decoded: RequestEnvelope<ExamplePayload> = ciborium::from_reader(buf.as_slice()).unwrap();
        assert_eq!(env, decoded);
    }

    #[cfg(feature = "std")]
    #[test]
    fn signing_message_is_deterministic() {
        let pk = [0x01; 32];
        let payload_hash = [0x02; 32];
        let sig_msg_1 = compute_signing_message(&pk, RequestType::QueryBalance, 5, 1776000000, &payload_hash);
        let sig_msg_2 = compute_signing_message(&pk, RequestType::QueryBalance, 5, 1776000000, &payload_hash);
        assert_eq!(sig_msg_1, sig_msg_2);
    }

    #[cfg(feature = "std")]
    #[test]
    fn signing_message_distinguishes_request_types() {
        let pk = [0x01; 32];
        let payload_hash = [0x02; 32];
        let a = compute_signing_message(&pk, RequestType::QueryBalance, 5, 1776000000, &payload_hash);
        let b = compute_signing_message(&pk, RequestType::QueryHistory, 5, 1776000000, &payload_hash);
        assert_ne!(a, b, "different request types must produce different sig messages");
    }

    #[cfg(feature = "std")]
    #[test]
    fn signing_message_distinguishes_nonces() {
        let pk = [0x01; 32];
        let payload_hash = [0x02; 32];
        let a = compute_signing_message(&pk, RequestType::QueryBalance, 5, 1776000000, &payload_hash);
        let b = compute_signing_message(&pk, RequestType::QueryBalance, 6, 1776000000, &payload_hash);
        assert_ne!(a, b, "different nonces must produce different sig messages");
    }

    #[cfg(feature = "std")]
    #[test]
    fn signing_message_distinguishes_timestamps() {
        let pk = [0x01; 32];
        let payload_hash = [0x02; 32];
        let a = compute_signing_message(&pk, RequestType::QueryBalance, 5, 1776000000, &payload_hash);
        let b = compute_signing_message(&pk, RequestType::QueryBalance, 5, 1776000001, &payload_hash);
        assert_ne!(a, b, "different timestamps must produce different sig messages");
    }
}
