//! AXIOM private-use CBOR tag allocations. See
//! `AXIOM_YellowPaper_Errors.md` §11.
//!
//! # Private-use range
//!
//! CBOR tags 65536 and above are reserved for private use without
//! IANA registration. AXIOM allocates a 32-bit sub-range within this
//! space:
//!
//! ```text
//! 0xAC10_00000000 .. 0xAC10_FFFFFFFF  — AXIOM protocol use
//! ```
//!
//! The `0xAC10` prefix is mnemonic ("AXIOM 10") and the 32-bit tail
//! gives 4 billion tag slots which is effectively infinite for a
//! single protocol's needs.
//!
//! # Sub-range allocation
//!
//! | Range | Purpose |
//! |---|---|
//! | `0xAC10_00000001` .. `0xAC10_0000FFFF` | `ErrorDetail` variants |
//! | `0xAC10_00010000` .. `0xAC10_0001FFFF` | `RecoveryHint` detail (reserved, unused) |
//! | `0xAC10_10000000` .. `0xAC10_1FFFFFFF` | Future reserved (response types, envelopes, etc.) |
//!
//! # Stability
//!
//! **Once a tag appears in a tagged release, it MUST NOT change.**
//! New variants get the next available tag. Deprecated variants keep
//! their tag forever (never reassigned).
//!
//! # Future IANA migration
//!
//! If AXIOM ever needs to interoperate at the CBOR level with external
//! tooling, these allocations can be registered with IANA post-hoc.
//! IANA accepts "grandfathered" private allocations that are already
//! in production use. The migration is transparent to clients.

/// AXIOM private-use CBOR tag prefix. All AXIOM tags are `AXIOM_TAG_BASE + offset`.
pub const AXIOM_TAG_BASE: u64 = 0xAC10_0000_0000;

// ── ErrorDetail variants ───────────────────────────────────────────────────
// Assignments are APPEND-ONLY. Never renumber. When adding a new variant,
// use AXIOM_TAG_BASE + (next available detail offset).

/// `ErrorDetail::StateChainMismatch`.
pub const TAG_STATE_CHAIN_MISMATCH: u64 = AXIOM_TAG_BASE + 0x0000_0001;

/// `ErrorDetail::SabrInsufficientOverlap`.
pub const TAG_SABR_INSUFFICIENT_OVERLAP: u64 = AXIOM_TAG_BASE + 0x0000_0002;

/// `ErrorDetail::Balance`.
pub const TAG_BALANCE: u64 = AXIOM_TAG_BASE + 0x0000_0003;

/// `ErrorDetail::ChequeBundle`.
pub const TAG_CHEQUE_BUNDLE: u64 = AXIOM_TAG_BASE + 0x0000_0004;

/// `ErrorDetail::VbcLifecycle`.
pub const TAG_VBC_LIFECYCLE: u64 = AXIOM_TAG_BASE + 0x0000_0005;

/// `ErrorDetail::RateLimit`.
pub const TAG_RATE_LIMIT: u64 = AXIOM_TAG_BASE + 0x0000_0006;

/// `ErrorDetail::WalletLock`.
pub const TAG_WALLET_LOCK: u64 = AXIOM_TAG_BASE + 0x0000_0007;

// Next available ErrorDetail offset: 0x0000_0008.
// When adding a new detail variant, use TAG_<NAME>: u64 = AXIOM_TAG_BASE + 0x0000_0008
// and increment this comment.

// ── Future: response types, envelopes, audit frames ────────────────────────
// Reserved range `0xAC10_10000000` .. `0xAC10_1FFFFFFF` for future use.
// Do not allocate from this range without adding a dedicated section here.

// ============================================================================
// Tag inclusion check
// ============================================================================

/// True iff the tag is inside AXIOM's private-use sub-range.
/// Used by deserializers to gate tag handling and reject unknown
/// foreign CBOR tags with a clean error instead of interpreting them.
pub const fn is_axiom_tag(tag: u64) -> bool {
    tag >= AXIOM_TAG_BASE && tag < AXIOM_TAG_BASE + 0x1_0000_0000
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axiom_tags_in_range() {
        assert!(is_axiom_tag(TAG_STATE_CHAIN_MISMATCH));
        assert!(is_axiom_tag(TAG_WALLET_LOCK));
        assert!(is_axiom_tag(AXIOM_TAG_BASE));
    }

    #[test]
    fn foreign_tags_out_of_range() {
        // Standard CBOR tags (0, 1, 55799) are outside AXIOM's range.
        assert!(!is_axiom_tag(0));
        assert!(!is_axiom_tag(1));
        assert!(!is_axiom_tag(55799));
        // Tag just below our base.
        assert!(!is_axiom_tag(AXIOM_TAG_BASE - 1));
        // Tag just above our range.
        assert!(!is_axiom_tag(AXIOM_TAG_BASE + 0x1_0000_0000));
    }

    #[test]
    fn detail_tags_are_distinct() {
        let tags = [
            TAG_STATE_CHAIN_MISMATCH,
            TAG_SABR_INSUFFICIENT_OVERLAP,
            TAG_BALANCE,
            TAG_CHEQUE_BUNDLE,
            TAG_VBC_LIFECYCLE,
            TAG_RATE_LIMIT,
            TAG_WALLET_LOCK,
        ];
        for (i, a) in tags.iter().enumerate() {
            for b in tags.iter().skip(i + 1) {
                assert_ne!(a, b, "duplicate tag allocation");
            }
        }
    }
}
