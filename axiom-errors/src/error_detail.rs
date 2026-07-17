//! Typed error detail variants. See `AXIOM_YellowPaper_Errors.md` §4
//! and §11 for the CBOR tag allocations.
//!
//! `ErrorDetail` is a discriminated union. On the wire it serializes
//! as a CBOR tagged value, where the tag identifies the variant and
//! the tagged payload is the variant's fields.

use alloc::string::String;
use serde::{Deserialize, Serialize};

// ============================================================================
// ErrorDetail enum
// ============================================================================

/// Structured, typed context for a specific error.
///
/// Not every `ErrorCode` has a detail; only the dispatch-critical
/// ones do. When no detail applies, `ErrorResponse.detail` is `None`.
///
/// Some fields are debug-gated — see `AXIOM_YellowPaper_Errors.md`
/// §7. The gate is enforced at the responding node; the type system
/// allows `None` to encode "not disclosed to this requester."
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "variant")]
pub enum ErrorDetail {
    /// State chain drift. Used for: E_SABR_HASH_MISMATCH,
    /// E_INVALID_STATE_ID, E_INVALID_WALLET_SEQ.
    StateChainMismatch(StateChainMismatchDetail),

    /// S-ABR fresh validator cannot accept without enough overlap sigs.
    /// Used for: E_SABR_INSUFFICIENT_OVERLAP.
    SabrInsufficientOverlap(SabrInsufficientOverlapDetail),

    /// Insufficient balance or balance math failure. Used for:
    /// E_INSUFFICIENT_BALANCE, E_REDEEM_BALANCE_OVERFLOW.
    Balance(BalanceDetail),

    /// Cheque bundle structural issue. Used for:
    /// E_CHEQUE_INCONSISTENT_BUNDLE, E_INSUFFICIENT_CHEQUES,
    /// E_FACT_DUPLICATE_WITNESS.
    ChequeBundle(ChequeBundleDetail),

    /// VBC lifecycle state. Used for: E_VBC_EXPIRED, E_VBC_NOT_YET_VALID.
    /// (Severity is an operator concern; clients see all VBC errors
    /// as the same category.)
    VbcLifecycle(VbcLifecycleDetail),

    /// Rate limit hit. Combined with `retry_after_secs` at the
    /// top-level ErrorResponse tells the client how long to wait and
    /// what the limit actually is.
    RateLimit(RateLimitDetail),

    /// Wallet is locked (banned, frozen, or in genesis lockup).
    /// `LockReason` discriminator tells the client whether the lock
    /// is reversible (S6 challenge) or time-gated (lockup).
    WalletLock(WalletLockDetail),
}

// ============================================================================
// Detail schemas
// ============================================================================

/// State chain drift detail. DEBUG-GATED fields are only disclosed
/// when the request is owner-authenticated via `RequestEnvelope`
/// (§12 of the Errors Yellow Paper) or the node is running in
/// `--debug-errors` mode. In production public endpoints, those
/// fields are `None`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateChainMismatchDetail {
    /// The state ID the client's TX declared as `consumed_state_id`.
    /// Always disclosed — the client submitted it, they know it.
    pub requested_consumed_state_id: [u8; 32],

    /// The wallet_seq the client's TX declared. Always disclosed.
    pub requested_wallet_seq: u64,

    /// The state ID the validator actually has stored for this wallet.
    /// DEBUG-GATED. `None` when not disclosed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validator_stored_state_id: Option<[u8; 32]>,

    /// The wallet_seq the validator has stored. DEBUG-GATED.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validator_wallet_seq: Option<u64>,
}

/// S-ABR fresh-validator overlap count detail.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SabrInsufficientOverlapDetail {
    /// How many overlap sigs the client included in the request.
    pub overlap_sigs_provided: u8,

    /// How many the validator requires (usually k-1 = 2 for k=3).
    pub overlap_sigs_required: u8,

    /// Whether this validator is treating itself as "fresh" (stored
    /// state does not match TX.consumed_state_id). If false, the
    /// rejection is for a different reason — unusual.
    pub is_fresh_validator: bool,
}

/// Balance detail. `current_balance` is DEBUG-GATED.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BalanceDetail {
    /// The amount the client's TX attempted to spend/move.
    pub requested_amount: u64,

    /// The wallet's actual balance at the validator. DEBUG-GATED —
    /// only disclosed to the wallet owner.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_balance: Option<u64>,
}

/// Cheque bundle detail. Tells the client exactly which field is
/// inconsistent so dedup or other fix can target the specific issue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChequeBundleDetail {
    /// Number of distinct validator IDs in the bundle (after dedup).
    pub distinct_validators: u8,

    /// The k required (3 for standard wallets, 5 for oracle, etc.)
    pub k_required: u8,

    /// If the bundle was inconsistent, which field failed the match.
    /// `None` if the bundle was just short on count.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub specific_field_mismatch: Option<BundleFieldMismatch>,
}

/// Which field in a cheque bundle failed the consistency check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum BundleFieldMismatch {
    /// Two cheques in the bundle reference different txids.
    Txid = 0,
    /// Two cheques have different sender_wallet_id fields.
    SenderWalletId = 1,
    /// Two cheques have different receiver_wallet_id fields.
    ReceiverWalletId = 2,
    /// Two cheques have different amount fields.
    Amount = 3,
    /// Two cheques have different epoch fields.
    Epoch = 4,
    /// Two cheques come from the same validator (duplicate).
    /// Fix: dedup by (txid, validator_id) per YP §17.9.4.0.
    DuplicateValidator = 5,
}

/// VBC lifecycle state. Both `E_VBC_EXPIRED` and `E_VBC_NOT_YET_VALID`
/// use this detail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct VbcLifecycleDetail {
    /// Tick at which this VBC expires (fixed in the credential).
    pub vbc_expires_at_tick: u64,

    /// The validator's current tick view.
    pub current_tick: u64,

    /// Signed: negative if expired, positive if not-yet-valid.
    /// `None` when the VBC is genuinely in a weird state (e.g.
    /// root-key mismatch — which uses a different error anyway).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticks_until_valid: Option<i64>,
}

/// Rate limit detail. Combined with the top-level `retry_after_secs`
/// field of `ErrorResponse`, fully informs the client how long to
/// wait.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RateLimitDetail {
    /// Max requests allowed in this window.
    pub limit: u32,
    /// Window length in seconds.
    pub window_secs: u32,
    /// Current count in the window at the time of rejection.
    pub current_count: u32,
}

/// Wallet lock detail (ban, freeze, or genesis lockup).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalletLockDetail {
    /// What kind of lock. Discriminator drives the SDK's UX dispatch.
    pub lock_reason: LockReason,

    /// Tick when the lock started.
    pub lock_started_at_tick: u64,

    /// Tick when the lock expires on its own. `None` for locks that
    /// don't auto-expire — e.g. S6DoubleSpendBan, which requires a
    /// challenge flow to reverse.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lock_expires_at_tick: Option<u64>,

    /// Path (HTTP route) to use for the challenge/recovery flow, if
    /// one exists. `Some("POST /ban-challenge")` for S6 bans.
    /// `None` for GenesisLockup (no recovery).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub challenge_path: Option<String>,
}

/// Why a wallet is locked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "reason")]
pub enum LockReason {
    /// Nabla detected a double-spend via gossip and banned the wallet
    /// (YPX-002 §3.3). Reversible via S6 challenge flow.
    S6DoubleSpendBan,

    /// A JFP vote approved freezing this wallet. Frozen until the
    /// JFP order is lifted or expires. `jfp_txid` identifies the
    /// approving JFP.
    JfpFreeze {
        jfp_txid: [u8; 32],
    },

    /// Genesis validator wallet inside the 3-year lockup period
    /// defined in White Paper §2.10.1. Not reversible; time-gated.
    GenesisLockup,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_chain_mismatch_cbor_roundtrip_with_debug_fields() {
        let d = ErrorDetail::StateChainMismatch(StateChainMismatchDetail {
            requested_consumed_state_id: [0x12; 32],
            requested_wallet_seq: 42,
            validator_stored_state_id: Some([0xab; 32]),
            validator_wallet_seq: Some(44),
        });
        let mut buf = Vec::new();
        ciborium::into_writer(&d, &mut buf).unwrap();
        let decoded: ErrorDetail = ciborium::from_reader(buf.as_slice()).unwrap();
        assert_eq!(d, decoded);
    }

    #[test]
    fn state_chain_mismatch_cbor_roundtrip_without_debug_fields() {
        let d = ErrorDetail::StateChainMismatch(StateChainMismatchDetail {
            requested_consumed_state_id: [0x34; 32],
            requested_wallet_seq: 100,
            validator_stored_state_id: None,
            validator_wallet_seq: None,
        });
        let mut buf = Vec::new();
        ciborium::into_writer(&d, &mut buf).unwrap();
        let decoded: ErrorDetail = ciborium::from_reader(buf.as_slice()).unwrap();
        assert_eq!(d, decoded);
    }

    #[test]
    fn cheque_bundle_duplicate_validator_roundtrip() {
        let d = ErrorDetail::ChequeBundle(ChequeBundleDetail {
            distinct_validators: 2,
            k_required: 3,
            specific_field_mismatch: Some(BundleFieldMismatch::DuplicateValidator),
        });
        let mut buf = Vec::new();
        ciborium::into_writer(&d, &mut buf).unwrap();
        let decoded: ErrorDetail = ciborium::from_reader(buf.as_slice()).unwrap();
        assert_eq!(d, decoded);
    }

    #[test]
    fn wallet_lock_s6_ban_roundtrip() {
        let d = ErrorDetail::WalletLock(WalletLockDetail {
            lock_reason: LockReason::S6DoubleSpendBan,
            lock_started_at_tick: 1000000,
            lock_expires_at_tick: None,
            challenge_path: Some(String::from("POST /ban-challenge")),
        });
        let mut buf = Vec::new();
        ciborium::into_writer(&d, &mut buf).unwrap();
        let decoded: ErrorDetail = ciborium::from_reader(buf.as_slice()).unwrap();
        assert_eq!(d, decoded);
    }

    #[test]
    fn wallet_lock_genesis_roundtrip() {
        let d = ErrorDetail::WalletLock(WalletLockDetail {
            lock_reason: LockReason::GenesisLockup,
            lock_started_at_tick: 1_773_878_400,
            lock_expires_at_tick: Some(1_868_486_400), // +3 years
            challenge_path: None,
        });
        let mut buf = Vec::new();
        ciborium::into_writer(&d, &mut buf).unwrap();
        let decoded: ErrorDetail = ciborium::from_reader(buf.as_slice()).unwrap();
        assert_eq!(d, decoded);
    }
}
