//! Error codes — stable string identifiers. See `AXIOM_YellowPaper_Errors.md` §6.
//!
//! **Stability contract:** once a code appears in a tagged release, it
//! MUST NOT change. New errors get new codes. Deprecated codes may be
//! marked `deprecated` but the string MUST NOT be reassigned.
//!
//! Format: `E_<DOMAIN>_<NAME>` in SCREAMING_SNAKE_CASE.
//!
//! # Code allocation by domain
//!
//! | Domain | Namespace prefix | Source enum |
//! |---|---|---|
//! | SABR / state chain | `E_SABR_*`, `E_STATE_*` | `ValidationError` |
//! | VBC | `E_VBC_*` | `ValidationError` |
//! | Cheque / bundle | `E_CHEQUE_*` | `ValidationError` |
//! | Balance / conservation | `E_INSUFFICIENT_*`, `E_*_BALANCE_*`, `E_*_AMOUNT` | `ValidationError` |
//! | FACT chain | `E_FACT_*` | `ValidationError` |
//! | Burn | `E_BURN_*` | `ValidationError` |
//! | Ark | `E_ARK_*` | `ValidationError` |
//! | Oracle | `E_ORACLE_*` | `ValidationError` / `OracleError` |
//! | Group wallet | `E_GROUP_*` | `ValidationError` |
//! | MVIB | `E_MVIB_*` | `ValidationError` |
//! | Console | `E_CONSOLE_*` | `ValidationError` |
//! | Fan-Out | `E_FANOUT_*` | `ValidationError` |
//! | CLARA | `E_CLARA_*` | `ValidationError` / `ClaraRegistrationError` |
//! | TXID attestation | `E_TXID_*` | `ValidationError` |
//! | Lambda-level | `E_LAMBDA_*` | `LambdaError` |
//! | Nabla-level | `E_NABLA_*` | `NablaError` |
//! | ANTIE-level | `E_ANTIE_*` | `AntieError` |
//! | §23.14 audit / AVM | `E_AVM_*` | `AvmError` |
//! | Request envelope (§12) | `E_REQUEST_*` | `axiom-errors` crate |

use alloc::string::String;
use serde::{Deserialize, Serialize};

/// A stable error code. Owned string on the wire; constructed from
/// the `&str` constants defined later in this module.
///
/// Errors are rare, so the extra allocation per error construction
/// is negligible compared to the simplicity of having an owned
/// serializable type. Use the constants (not raw strings) at call
/// sites to get the stability guarantee.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ErrorCode(pub String);

impl ErrorCode {
    /// Wrap a `&str` constant in an owned `ErrorCode`.
    /// Preferred usage: `ErrorCode::from_static(E_SABR_HASH_MISMATCH)`.
    pub fn from_static(s: &str) -> Self {
        Self(String::from(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ErrorCode {
    fn from(s: &str) -> Self {
        Self::from_static(s)
    }
}

impl core::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ============================================================================
// Code constants
// ============================================================================
// These are the canonical set. Add new constants here; grep for this module
// to find all known codes.
//
// Where `ValidationError::Display` already emits a code string, we use the
// SAME string here — no renaming. That's why many E_ codes have no "SABR" /
// "CHEQUE" namespace prefix: they date from before this taxonomy existed.

// ── State / sequence (ValidationError) ─────────────────────────────────────
pub const E_STATE_ID_CONSUMED: &str = "E_STATE_ID_CONSUMED";
pub const E_INVALID_STATE_ID: &str = "E_INVALID_STATE_ID";
pub const E_INVALID_WALLET_SEQ: &str = "E_INVALID_WALLET_SEQ";
pub const E_WALLET_SEQ_OVERFLOW: &str = "E_WALLET_SEQ_OVERFLOW";
/// §15 anchor check: client `current_state` did not re-derive to the
/// k-signed `prev_receipt.state_hash`. Carries `RecoveryHint::ClaraHealNextSend`.
pub const E_STATE_NOT_ANCHORED: &str = "E_STATE_NOT_ANCHORED";
/// Fable review 2026-10-01 F-1(b): a CL5 redeem's DECLARED receiver state and
/// the carried `prev_receipts` do not have the anchorable shape — a non-zero
/// (returning) receiver state must ship EXACTLY ONE receipt (the wallet's last),
/// and a first-time zero state must ship NONE. A receipt that IS carried but does
/// not re-derive the declared state is `E_STATE_NOT_ANCHORED` (heal hint applies).
pub const E_RECEIVER_STATE_NOT_ANCHORED: &str = "E_RECEIVER_STATE_NOT_ANCHORED";
pub const E_INVALID_WALLET_ID: &str = "E_INVALID_WALLET_ID";
pub const E_MALFORMED_ADDRESS: &str = "E_MALFORMED_ADDRESS";
pub const E_SENDER_WALLET_ID_MISMATCH: &str = "E_SENDER_WALLET_ID_MISMATCH";
pub const E_MISSING_WALLET_STATE: &str = "E_MISSING_WALLET_STATE";

// ── Signatures ─────────────────────────────────────────────────────────────
pub const E_INVALID_CLIENT_SIG: &str = "E_INVALID_CLIENT_SIG";
pub const E_INVALID_WITNESS_SIG: &str = "E_INVALID_WITNESS_SIG";
pub const E_UNSUPPORTED_SIG_ALG: &str = "E_UNSUPPORTED_SIG_ALG";

// ── Balance / conservation ─────────────────────────────────────────────────
pub const E_INSUFFICIENT_BALANCE: &str = "E_INSUFFICIENT_BALANCE";
pub const E_CONSERVATION_VIOLATION: &str = "E_CONSERVATION_VIOLATION";
pub const E_ZERO_AMOUNT: &str = "E_ZERO_AMOUNT";
pub const E_DUST_AMOUNT: &str = "E_DUST_AMOUNT";

// ── VBC ────────────────────────────────────────────────────────────────────
pub const E_INVALID_VBC: &str = "E_INVALID_VBC";
pub const E_VBC_EXPIRED: &str = "E_VBC_EXPIRED";
pub const E_VBC_NOT_YET_VALID: &str = "E_VBC_NOT_YET_VALID";
/// KI#130 — a witness VBC is within `vbc_unusable_remaining_ticks` of its hard
/// expiry (judged on the attested tick): it "loses usefulness" before it expires,
/// so it may no longer serve as a witness. Retrying a fresher validator fixes it.
pub const E_VBC_UNUSABLE_SOON: &str = "E_VBC_UNUSABLE_SOON";
/// KI#130 Gap B — the round's attested OODS tick is staler than the wallet's last
/// round by more than the carrier-latency buffer: a replay of an old reading. Fetch
/// a fresh OODS reading (or retry a different validator) and resend.
pub const E_VBC_STALE_ATTESTATION: &str = "E_VBC_STALE_ATTESTATION";
/// KI#130 — a VBC-issuance request (CL8) asks for a lifetime (`expires_at - issued_at`)
/// longer than the max non-genesis validity. Malformed request; request a shorter
/// `expires_at`. (Genesis 10-year certs are ceremony-signed, never via CL8.)
pub const E_VBC_LIFETIME_TOO_LONG: &str = "E_VBC_LIFETIME_TOO_LONG";
/// A PROVISIONAL cert (ValidatorJoin §5.2.2) was presented as a signer's
/// credential. Not a lifecycle timing problem — the cert is doing a job it
/// was never issued for, and waiting does not fix it.
pub const E_VBC_PROVISIONAL_CANNOT_SERVE: &str = "E_VBC_PROVISIONAL_CANNOT_SERVE";
// ── §5.3 genesis-lineage admission (ValidatorJoin §5.3) ───────────────────
// One vocabulary for BOTH sites that implement the rule: the enforcement in
// `vbc::verify_chain_recursive` and the fail-fast twin in CL8. Before these
// existed every arm answered E_INVALID_VBC, which made a refused certificate
// request undiagnosable from outside the guest (RULE 3 shape 2).
pub const E_VBC_NO_ATTESTED_TICK: &str = "E_VBC_NO_ATTESTED_TICK";
pub const E_VBC_ISSUER_CERT_MISSING: &str = "E_VBC_ISSUER_CERT_MISSING";
pub const E_VBC_ISSUER_CANNOT_ISSUE: &str = "E_VBC_ISSUER_CANNOT_ISSUE";
pub const E_VBC_ISSUER_NO_LINEAGE: &str = "E_VBC_ISSUER_NO_LINEAGE";
pub const E_VBC_ISSUERS_SHARE_LINEAGE: &str = "E_VBC_ISSUERS_SHARE_LINEAGE";
pub const E_VBC_LINEAGE_NOT_ADOPTED: &str = "E_VBC_LINEAGE_NOT_ADOPTED";

// ── Q2-b renewal proof-of-validation (the owner ruled 2026-09-21) ─────────────
// A VBC RENEWAL must present >=1 k-signed receipt the renewing validator
// co-signed this term — proof it did real witnessing work. Verified in-guest
// by CL8 `verify_renewal_work_receipt`; no Nabla (RULE 7). Distinct codes per
// RULE 3 so a refused renewal is diagnosable from outside the guest.
pub const E_VBC_RENEWAL_NO_PROOF_OF_WORK: &str = "E_VBC_RENEWAL_NO_PROOF_OF_WORK";
pub const E_VBC_RENEWAL_NOT_CO_SIGNED: &str = "E_VBC_RENEWAL_NOT_CO_SIGNED";
pub const E_VBC_RENEWAL_WORK_RECEIPT_STALE: &str = "E_VBC_RENEWAL_WORK_RECEIPT_STALE";
pub const E_VBC_RENEWAL_WORK_RECEIPT_SUB_QUORUM: &str = "E_VBC_RENEWAL_WORK_RECEIPT_SUB_QUORUM";

// ── CL8 certificate issuance ──────────────────────────────────────────────
pub const E_CL8_MISSING_BUNDLE: &str = "E_CL8_MISSING_BUNDLE";
pub const E_CL8_MISSING_ISSUER_KEY: &str = "E_CL8_MISSING_ISSUER_KEY";
pub const E_CL8_SIGNER_NOT_IN_ISSUER_SET: &str = "E_CL8_SIGNER_NOT_IN_ISSUER_SET";
pub const E_CL8_PROVISIONAL_LIFETIME_INVALID: &str = "E_CL8_PROVISIONAL_LIFETIME_INVALID";
pub const E_CL8_ISSUER_KEY_UNUSABLE: &str = "E_CL8_ISSUER_KEY_UNUSABLE";
pub const E_CL8_SIGNING_FAILED: &str = "E_CL8_SIGNING_FAILED";
pub const E_CL8_VERIFY_AFTER_SIGN_FAILED: &str = "E_CL8_VERIFY_AFTER_SIGN_FAILED";
pub const E_CL8_OODS_STAMP_MISMATCH: &str = "E_CL8_OODS_STAMP_MISMATCH";

pub const E_VBC_CHAIN_TOO_DEEP: &str = "E_VBC_CHAIN_TOO_DEEP";
pub const E_VBC_MISSING_ISSUER: &str = "E_VBC_MISSING_ISSUER";
pub const E_VBC_ROOT_KEY_MISMATCH: &str = "E_VBC_ROOT_KEY_MISMATCH";
pub const E_DUPLICATE_VALIDATOR: &str = "E_DUPLICATE_VALIDATOR";
pub const E_INVALID_VBC_COUNT: &str = "E_INVALID_VBC_COUNT";
pub const E_GENESIS_NAME_RESERVED: &str = "E_GENESIS_NAME_RESERVED";

// ── Cheque / redeem ────────────────────────────────────────────────────────
pub const E_INSUFFICIENT_CHEQUES: &str = "E_INSUFFICIENT_CHEQUES";
pub const E_CHEQUE_INCONSISTENT_BUNDLE: &str = "E_CHEQUE_INCONSISTENT_BUNDLE";
pub const E_INVALID_CHEQUE_SIG: &str = "E_INVALID_CHEQUE_SIG";
pub const E_CHEQUE_ALREADY_REDEEMED: &str = "E_CHEQUE_ALREADY_REDEEMED";
pub const E_REDEEM_BALANCE_MISMATCH: &str = "E_REDEEM_BALANCE_MISMATCH";
pub const E_REDEEM_BALANCE_OVERFLOW: &str = "E_REDEEM_BALANCE_OVERFLOW";
pub const E_MISSING_EXECUTION_PROOF: &str = "E_MISSING_EXECUTION_PROOF";
pub const E_MISSING_REDEEM_INPUTS: &str = "E_MISSING_REDEEM_INPUTS";
pub const E_MISSING_VBC: &str = "E_MISSING_VBC";

// ── TXID attestation ───────────────────────────────────────────────────────
pub const E_TXID_ATTESTATION_MISSING: &str = "E_TXID_ATTESTATION_MISSING";
pub const E_TXID_ATTESTATION_INVALID_SIG: &str = "E_TXID_ATTESTATION_INVALID_SIG";
pub const E_TXID_ATTESTATION_REDEEMED: &str = "E_TXID_ATTESTATION_REDEEMED";
pub const E_TXID_ATTESTATION_BAD_STATUS: &str = "E_TXID_ATTESTATION_BAD_STATUS";
pub const E_TXID_ATTESTATION_UNTRUSTED: &str = "E_TXID_ATTESTATION_UNTRUSTED";
pub const E_TXID_PHASED_OUT: &str = "E_TXID_PHASED_OUT";

// ── Cheque claim proof (synchronous double-redeem prevention) ──────────────
// Nabla writer signs `compute::redeem_claim_nabla_payload` = BLAKE3(
// "AXIOM_REDEEM_CLAIM" || cheque_id || "CLAIMED" || tick_le || claim_sig) on
// successful register_cheque_claim. Closes the gossip-race window the older
// txid_attestation path leaves open: writer is single-source, second register
// returns CONFLICT, attacker gets no signed proof. Since YPX-022 §2.1.2a
// (KI#205, 2026-09-25) the claim itself is AUTHENTICATED: `claim_sig` is the
// claimant's Ed25519 signature over `compute::cheque_claim_signing_payload`, and
// the Nabla signature covers it. See nabla/src/bin/nabla_node.rs::register_cheque_claim_core.
pub const E_CHEQUE_CLAIM_PROOF_MISSING: &str = "E_CHEQUE_CLAIM_PROOF_MISSING";
pub const E_CHEQUE_CLAIM_PROOF_INVALID_SIG: &str = "E_CHEQUE_CLAIM_PROOF_INVALID_SIG";
/// YPX-022 §2.1.2a (KI#205): the proof's `claim_sig` does not verify by
/// `client_pk` over `cheque_claim_signing_payload(cheque_id, client_pk, k_tier,
/// wallet_address)` — the claim was not made by the key the cheque is addressed to.
pub const E_CHEQUE_CLAIM_PROOF_UNAUTHENTICATED: &str = "E_CHEQUE_CLAIM_PROOF_UNAUTHENTICATED";
pub const E_CHEQUE_CLAIM_PROOF_TXID_MISMATCH: &str = "E_CHEQUE_CLAIM_PROOF_TXID_MISMATCH";
pub const E_CHEQUE_CLAIM_PROOF_RECEIVER_MISMATCH: &str = "E_CHEQUE_CLAIM_PROOF_RECEIVER_MISMATCH";
pub const E_CHEQUE_CLAIM_PROOF_UNTRUSTED: &str = "E_CHEQUE_CLAIM_PROOF_UNTRUSTED";
pub const E_TXID_ALREADY_IN_RECEIVER_CHAIN: &str = "E_TXID_ALREADY_IN_RECEIVER_CHAIN";
/// Claim proof's tick is outside the freshness window.  Either older
/// than `cheque_claim_proof_max_age_ticks` (protocol_core.toml, 17_280) or
/// futureshifted past TICK_SLACK. This is CL5's FRESHNESS bound on the proof
/// only — Nabla holds the claim itself until `recall_init_window_high`
/// (YPX-022 §2.1.2a item 2); the same key may re-claim at any time.
pub const E_CHEQUE_CLAIM_PROOF_EXPIRED: &str = "E_CHEQUE_CLAIM_PROOF_EXPIRED";

// ── S-ABR ──────────────────────────────────────────────────────────────────
pub const E_SABR_INSUFFICIENT_OVERLAP: &str = "E_SABR_INSUFFICIENT_OVERLAP";
pub const E_SABR_OVERLAP_NOT_IN_PREV: &str = "E_SABR_OVERLAP_NOT_IN_PREV";
pub const E_SABR_MISSING_VALIDATOR_PK: &str = "E_SABR_MISSING_VALIDATOR_PK";
pub const E_SABR_HASH_MISMATCH: &str = "E_SABR_HASH_MISMATCH";

// ── FACT chain ─────────────────────────────────────────────────────────────
pub const E_FACT_CHAIN_TOO_DEEP: &str = "E_FACT_CHAIN_TOO_DEEP";
pub const E_FACT_CHAIN_BREAK: &str = "E_FACT_CHAIN_BREAK";
pub const E_FACT_INSUFFICIENT_WITNESSES: &str = "E_FACT_INSUFFICIENT_WITNESSES";
pub const E_FACT_INVALID_SIG: &str = "E_FACT_INVALID_SIG";
pub const E_FACT_DUPLICATE_WITNESS: &str = "E_FACT_DUPLICATE_WITNESS";
pub const E_FACT_INVALID_CHECKPOINT: &str = "E_FACT_INVALID_CHECKPOINT";
pub const E_FACT_CHAIN_EMPTY: &str = "E_FACT_CHAIN_EMPTY";
pub const E_FACT_AMOUNT_OVERFLOW: &str = "E_FACT_AMOUNT_OVERFLOW";
// YP §26.17.6.5 FACT Provenance Binding (2026-09-11, KI#145)
pub const E_FACT_WITNESS_UNCERTIFIED: &str = "E_FACT_WITNESS_UNCERTIFIED";
pub const E_FACT_ORIGIN_INVALID: &str = "E_FACT_ORIGIN_INVALID";
pub const E_FACT_CERTIFICATE_INVALID: &str = "E_FACT_CERTIFICATE_INVALID";
pub const E_FACT_BURN_SIG_INVALID: &str = "E_FACT_BURN_SIG_INVALID";
/// KI#152 (b), 2026-09-13: a subsidised stake claim addressed to a non-Standard
/// tier of the claimant's own key (code 1011).
pub const E_STAKE_CLAIM_TIER_INVALID: &str = "E_STAKE_CLAIM_TIER_INVALID";

// ── Burn ───────────────────────────────────────────────────────────────────
pub const E_BURN_NO_FACT_CHAIN: &str = "E_BURN_NO_FACT_CHAIN";
pub const E_BURN_MISSING_TARGET: &str = "E_BURN_MISSING_TARGET";
pub const E_BURN_TARGET_NOT_FOUND: &str = "E_BURN_TARGET_NOT_FOUND";
pub const E_BURN_TARGET_NOT_SCARRED: &str = "E_BURN_TARGET_NOT_SCARRED";
pub const E_BURN_TARGET_ALREADY_BURNED: &str = "E_BURN_TARGET_ALREADY_BURNED";
pub const E_BURN_AMOUNT_MISMATCH: &str = "E_BURN_AMOUNT_MISMATCH";
pub const E_BURN_PROOF_INSUFFICIENT_WITNESSES: &str = "E_BURN_PROOF_INSUFFICIENT_WITNESSES";
pub const E_BURN_PROOF_DUPLICATE_VALIDATOR: &str = "E_BURN_PROOF_DUPLICATE_VALIDATOR";
pub const E_BURN_TX_ID_NOT_IN_CHAIN: &str = "E_BURN_TX_ID_NOT_IN_CHAIN";
pub const E_BURN_TARGET_MISMATCH: &str = "E_BURN_TARGET_MISMATCH";
pub const E_TOO_MANY_UNRESOLVED_SCARS: &str = "E_TOO_MANY_UNRESOLVED_SCARS";

// ── Ark ────────────────────────────────────────────────────────────────────
pub const E_ARK_TO_NON_ARK: &str = "E_ARK_TO_NON_ARK";
pub const E_ARK_CHARGE_NOT_OWNER: &str = "E_ARK_CHARGE_NOT_OWNER";
pub const E_ARK_UNLOAD_SCARRED: &str = "E_ARK_UNLOAD_SCARRED";
/// §11.9.1b — a CHARGE into the owner's own Ark wallet is refused while the
/// SENDING (normal) wallet carries an unresolved FACT link. The owner's ruling
/// 2026-09-17: "sender needs to be healthy when sending between a self send
/// normal and ark" — the constraint is on the SENDER in BOTH directions; the
/// receiver never needs to be clean.
pub const E_ARK_CHARGE_SCARRED: &str = "E_ARK_CHARGE_SCARRED";
pub const E_SELF_SEND_REJECTED: &str = "E_SELF_SEND_REJECTED";
pub const E_RECEIVER_ADDRESS_REQUIRED: &str = "E_RECEIVER_ADDRESS_REQUIRED";
pub const E_INVALID_RECEIVER_ADDRESS: &str = "E_INVALID_RECEIVER_ADDRESS";

// ── Stake / freeze / lockup ────────────────────────────────────────────────
pub const E_STAKE_WALLET_MISMATCH: &str = "E_STAKE_WALLET_MISMATCH";
pub const E_STAKE_NABLA_SIG_INVALID: &str = "E_STAKE_NABLA_SIG_INVALID";
pub const E_STAKE_STATE_MISMATCH: &str = "E_STAKE_STATE_MISMATCH";
pub const E_STAKE_INSUFFICIENT_RECEIPTS: &str = "E_STAKE_INSUFFICIENT_RECEIPTS";
pub const E_STAKE_PROOF_EXPIRED: &str = "E_STAKE_PROOF_EXPIRED";
pub const E_WALLET_FROZEN: &str = "E_WALLET_FROZEN";
pub const E_GENESIS_STAKE_LOCKED: &str = "E_GENESIS_STAKE_LOCKED";
/// §5.2.2c — a SUBSIDISED validator stake lock (`WalletState.wall_clock_lock`)
/// has not expired. The general-purpose sibling of `E_GENESIS_STAKE_LOCKED`,
/// which is the hardcoded genesis lockup; both are "this wallet cannot send
/// before T", and neither has a recovery beyond waiting.
pub const E_STAKE_LOCKED: &str = "E_STAKE_LOCKED";
/// §5.2.2c INTERLOCK (KI#137) — the wallet's two stake deadlines are a distance
/// apart that NO tier could have minted, so the pair is forged rather than
/// merely still-running. Distinct from `E_STAKE_LOCKED`, which means "the lock
/// is real and has not expired": this one means "this lock is not real".
pub const E_STAKE_LOCK_PAIR_UNMINTABLE: &str = "E_STAKE_LOCK_PAIR_UNMINTABLE";
/// ValidatorJoin §6b.13 (KI#225) — the STAKE FLOOR: a debit would take a wallet
/// below `VALIDATOR_STAKE_FLOOR_ATOMS` (500 AXC) while its `stake_floor_until`
/// is live (a certificate from its own `VbcRequest` can still be live). Its own
/// code, not `E_STAKE_LOCKED`: the surplus above 500 moves freely — send less,
/// or wait for the floor to lapse.
pub const E_STAKE_FLOOR: &str = "E_STAKE_FLOOR";
/// ValidatorJoin §6b.13 — the wallet state's format block is not the current
/// one (`wallet_version != WALLET_VERSION`, or a reserved ext field non-zero).
pub const E_WALLET_FORMAT_INVALID: &str = "E_WALLET_FORMAT_INVALID";
/// §5.2.2c — the claimant's declared epoch and the ISSUERS' signed `created_at`
/// disagree about the claim's own witness round by more than the tolerated range.
/// One of them is lying; neither is trusted alone, so the claim is refused.
pub const E_STAKE_LOCK_TIME_DISAGREEMENT: &str = "E_STAKE_LOCK_TIME_DISAGREEMENT";
/// §23.15 TRANSACTION VELOCITY LIMIT (TVL, KI#221) — this transaction's
/// Nabla-attested tick is fewer than `TX_VELOCITY_MIN_TICKS` after the wallet's
/// previous witnessed transaction. Refused so a wallet cannot burst two forked
/// sends faster than the conflict-ban gossip converges. Retryable: WaitAndRetry
/// once the tick floor elapses (~25 s).
pub const E_TX_VELOCITY_TOO_FAST: &str = "E_TX_VELOCITY_TOO_FAST";
pub const E_NABLA_WRITER_DETECTED: &str = "E_NABLA_WRITER_DETECTED";

// ── CLARA ──────────────────────────────────────────────────────────────────
pub const E_CLARA_INVALID_SIGNATURE: &str = "E_CLARA_INVALID_SIGNATURE";
pub const E_CLARA_WALLET_PK_MISMATCH: &str = "E_CLARA_WALLET_PK_MISMATCH";
pub const E_CLARA_STATE_NOT_GARBAGE: &str = "E_CLARA_STATE_NOT_GARBAGE";
pub const E_CLARA_NBC_TRUST_FAILED: &str = "E_CLARA_NBC_TRUST_FAILED";
pub const E_CLARA_EMPTY_GARBAGE: &str = "E_CLARA_EMPTY_GARBAGE";

// ── Oracle (YPX-012) ──────────────────────────────────────────────────
pub const E_ORACLE_SENDER_MISMATCH: &str = "E_ORACLE_SENDER_MISMATCH";
pub const E_ORACLE_INSUFFICIENT_K: &str = "E_ORACLE_INSUFFICIENT_K";
pub const E_ORACLE_VBC_TOO_OLD: &str = "E_ORACLE_VBC_TOO_OLD";
pub const E_ORACLE_INSUFFICIENT_STAKE: &str = "E_ORACLE_INSUFFICIENT_STAKE";
pub const E_ORACLE_STAKE_SCARRED: &str = "E_ORACLE_STAKE_SCARRED";
pub const E_ORACLE_PLATFORM_INVALID: &str = "E_ORACLE_PLATFORM_INVALID";
pub const E_ORACLE_LIVING_SIG_MISSING: &str = "E_ORACLE_LIVING_SIG_MISSING";
pub const E_ORACLE_ZERO_DELTA: &str = "E_ORACLE_ZERO_DELTA";
pub const E_ORACLE_NONZERO_AMOUNT: &str = "E_ORACLE_NONZERO_AMOUNT";
pub const E_ORACLE_MATURITY_NOT_REACHED: &str = "E_ORACLE_MATURITY_NOT_REACHED";

// ── Group wallet ──────────────────────────────────────────────────────
pub const E_GROUP_TOO_MANY_MEMBERS: &str = "E_GROUP_TOO_MANY_MEMBERS";
pub const E_GROUP_SHARE_BPS_INVALID: &str = "E_GROUP_SHARE_BPS_INVALID";
pub const E_GROUP_NOT_MEMBER: &str = "E_GROUP_NOT_MEMBER";
pub const E_GROUP_INSUFFICIENT_AVAILABLE: &str = "E_GROUP_INSUFFICIENT_AVAILABLE";
pub const E_GROUP_CHECKSUM_FAILED: &str = "E_GROUP_CHECKSUM_FAILED";
pub const E_GROUP_MEMBERS_IMMUTABLE: &str = "E_GROUP_MEMBERS_IMMUTABLE";
pub const E_GROUP_DISTRIBUTION_OVERFLOW: &str = "E_GROUP_DISTRIBUTION_OVERFLOW";
pub const E_GROUP_MEMBER_MISMATCH: &str = "E_GROUP_MEMBER_MISMATCH";

// ── MVIB ──────────────────────────────────────────────────────────────
pub const E_MVIB_EMPTY_ADMISSION_SET: &str = "E_MVIB_EMPTY_ADMISSION_SET";
pub const E_MVIB_INVALID_ADMISSION_SET_SIZE: &str = "E_MVIB_INVALID_ADMISSION_SET_SIZE";
pub const E_MVIB_DUPLICATE_ISSUER: &str = "E_MVIB_DUPLICATE_ISSUER";
pub const E_MVIB_INVALID_SIGNATURE: &str = "E_MVIB_INVALID_SIGNATURE";
pub const E_MVIB_INVALID_TICK: &str = "E_MVIB_INVALID_TICK";

// ── Console (YPX-013) ────────────────────────────────────────────────
pub const E_CONSOLE_INVALID_GENERATION: &str = "E_CONSOLE_INVALID_GENERATION";
pub const E_CONSOLE_CHAIN_MISMATCH: &str = "E_CONSOLE_CHAIN_MISMATCH";
pub const E_CONSOLE_INVALID_SEAT_COUNT: &str = "E_CONSOLE_INVALID_SEAT_COUNT";
pub const E_CONSOLE_DUPLICATE_SEAT: &str = "E_CONSOLE_DUPLICATE_SEAT";
pub const E_CONSOLE_TERM_MISMATCH: &str = "E_CONSOLE_TERM_MISMATCH";
pub const E_CONSOLE_INVALID_TERM_LENGTH: &str = "E_CONSOLE_INVALID_TERM_LENGTH";
pub const E_CONSOLE_INVALID_SELECTOR: &str = "E_CONSOLE_INVALID_SELECTOR";
pub const E_CONSOLE_INVALID_PICK: &str = "E_CONSOLE_INVALID_PICK";
pub const E_CONSOLE_INCOMPLETE_SELECTION: &str = "E_CONSOLE_INCOMPLETE_SELECTION";
pub const E_CONSOLE_NOT_MEMBER: &str = "E_CONSOLE_NOT_MEMBER";
pub const E_CONSOLE_PHASE_OUT_INVALID: &str = "E_CONSOLE_PHASE_OUT_INVALID";

// ── Fan-Out (CL10) ───────────────────────────────────────────────────
pub const E_FANOUT_MISSING_MESSAGE: &str = "E_FANOUT_MISSING_MESSAGE";
pub const E_FANOUT_TTL_EXCEEDED: &str = "E_FANOUT_TTL_EXCEEDED";
pub const E_FANOUT_INVALID_FANOUT: &str = "E_FANOUT_INVALID_FANOUT";
pub const E_FANOUT_CONTENT_EMPTY: &str = "E_FANOUT_CONTENT_EMPTY";
pub const E_FANOUT_CONTENT_TOO_LARGE: &str = "E_FANOUT_CONTENT_TOO_LARGE";
pub const E_FANOUT_TTL_EXPIRED: &str = "E_FANOUT_TTL_EXPIRED";
pub const E_FANOUT_TTL_INFLATED: &str = "E_FANOUT_TTL_INFLATED";
pub const E_FANOUT_UNKNOWN_CONTENT_TYPE: &str = "E_FANOUT_UNKNOWN_CONTENT_TYPE";
pub const E_FANOUT_TIMESTAMP_FUTURE: &str = "E_FANOUT_TIMESTAMP_FUTURE";
pub const E_FANOUT_TIMESTAMP_EXPIRED: &str = "E_FANOUT_TIMESTAMP_EXPIRED";
pub const E_FANOUT_DIFFUSION_ID_MISMATCH: &str = "E_FANOUT_DIFFUSION_ID_MISMATCH";
pub const E_FANOUT_INVALID_ORIGINATOR: &str = "E_FANOUT_INVALID_ORIGINATOR";
pub const E_FANOUT_ORIGINATOR_PK_MISMATCH: &str = "E_FANOUT_ORIGINATOR_PK_MISMATCH";
pub const E_FANOUT_INVALID_SIGNATURE: &str = "E_FANOUT_INVALID_SIGNATURE";

// ── Heal ──────────────────────────────────────────────────────────────
pub const E_HEAL_NOT_NEEDED: &str = "E_HEAL_NOT_NEEDED";

// ── Cheque / fee (additional) ─────────────────────────────────────────
pub const E_INCONSISTENT_CHEQUE_BUNDLE: &str = "E_INCONSISTENT_CHEQUE_BUNDLE";
pub const E_FEE_REDEMPTION_CONFLICT: &str = "E_FEE_REDEMPTION_CONFLICT";
pub const E_INVALID_FEE_CHEQUE: &str = "E_INVALID_FEE_CHEQUE";
pub const E_INVALID_CONFIRMATION_CHEQUE: &str = "E_INVALID_CONFIRMATION_CHEQUE";
pub const E_MISSING_FEE_CHEQUES: &str = "E_MISSING_FEE_CHEQUES";
pub const E_INSUFFICIENT_FEE_WITNESSES: &str = "E_INSUFFICIENT_FEE_WITNESSES";

// ── Validation misc (additional Core codes) ───────────────────────────
pub const E_MISSING_PREV_RECEIPTS: &str = "E_MISSING_PREV_RECEIPTS";
pub const E_INVALID_GENESIS_TX: &str = "E_INVALID_GENESIS_TX";
pub const E_INVALID_EXECUTION_PROOF: &str = "E_INVALID_EXECUTION_PROOF";
pub const E_PROGRAM_DIGEST_MISMATCH: &str = "E_PROGRAM_DIGEST_MISMATCH";
pub const E_INVALID_JSON: &str = "E_INVALID_JSON";
pub const E_CARRIERS_TOO_LARGE: &str = "E_CARRIERS_TOO_LARGE";
pub const E_INVALID_HINT_COUNT: &str = "E_INVALID_HINT_COUNT";
pub const E_SELF_HINT_NOT_ALLOWED: &str = "E_SELF_HINT_NOT_ALLOWED";
pub const E_ARK_NOT_IMPLEMENTED: &str = "E_ARK_NOT_IMPLEMENTED";
pub const E_REFERENCE_TOO_LARGE: &str = "E_REFERENCE_TOO_LARGE";
pub const E_INVALID_MODE: &str = "E_INVALID_MODE";
pub const E_INTERNAL: &str = "E_INTERNAL";
pub const E_RECEIPT_WRONG_WORLDLINE: &str = "E_RECEIPT_WRONG_WORLDLINE";
pub const E_RECEIPT_LINEAGE_MISMATCH: &str = "E_RECEIPT_LINEAGE_MISMATCH";
pub const E_VERSION_MISMATCH: &str = "E_VERSION_MISMATCH";
// E_MISSING_DILITHIUM_KEY / E_MISSING_DILITHIUM_PK removed 2026-09-15 with Core CL9.
pub const E_MISSING_FIELD: &str = "E_MISSING_FIELD";
pub const E_WALLET_SECRET_MISMATCH: &str = "E_WALLET_SECRET_MISMATCH";
pub const E_INSUFFICIENT_STAKE: &str = "E_INSUFFICIENT_STAKE";
pub const E_REDEEM_REGISTRATION_INCOMPLETE: &str = "E_REDEEM_REGISTRATION_INCOMPLETE";
pub const E_GENESIS_CLAIM_INVALID_SEQ: &str = "E_GENESIS_CLAIM_INVALID_SEQ";
pub const E_GENESIS_CLAIM_NON_ZERO_AMOUNT: &str = "E_GENESIS_CLAIM_NON_ZERO_AMOUNT";

// ── FACT chain (additional) ───────────────────────────────────────────
pub const E_FACT_INVALID_SIGNATURE: &str = "E_FACT_INVALID_SIGNATURE";

// `E_AUTH_HASH_REQUIRED` / `E_INVALID_AUTH_PROOF` DELETED 2026-09-25 with
// `Transaction.owner_proof` (KI#108).

// ── Lambda-level errors ────────────────────────────────────────────────────
pub const E_LAMBDA_INSUFFICIENT_WITNESSES: &str = "E_LAMBDA_INSUFFICIENT_WITNESSES";
pub const E_LAMBDA_STORAGE_ERROR: &str = "E_LAMBDA_STORAGE_ERROR";
pub const E_LAMBDA_WALLET_NOT_FOUND: &str = "E_LAMBDA_WALLET_NOT_FOUND";
pub const E_LAMBDA_DUPLICATE_TRANSACTION: &str = "E_LAMBDA_DUPLICATE_TRANSACTION";
pub const E_LAMBDA_CONSENSUS_TIMEOUT: &str = "E_LAMBDA_CONSENSUS_TIMEOUT";
pub const E_LAMBDA_CONFIG_ERROR: &str = "E_LAMBDA_CONFIG_ERROR";
pub const E_LAMBDA_INVALID_REQUEST: &str = "E_LAMBDA_INVALID_REQUEST";
pub const E_LAMBDA_IO_ERROR: &str = "E_LAMBDA_IO_ERROR";
pub const E_LAMBDA_SERIALIZATION_ERROR: &str = "E_LAMBDA_SERIALIZATION_ERROR";
pub const E_LAMBDA_RATE_LIMIT_EXCEEDED: &str = "E_LAMBDA_RATE_LIMIT_EXCEEDED";
pub const E_LAMBDA_SCAR_CONSENT_REQUIRED: &str = "E_LAMBDA_SCAR_CONSENT_REQUIRED";
pub const E_LAMBDA_INVALID_SCAR_PASSCODE: &str = "E_LAMBDA_INVALID_SCAR_PASSCODE";
pub const E_LAMBDA_WALLET_FROZEN_JFP: &str = "E_LAMBDA_WALLET_FROZEN_JFP";

// Phase 2b.5 — Lambda ack/redeem/fee-redemption-path ad-hoc rejects.
// These cover the ~30 static-string error sites in consensus.rs that
// don't map cleanly to a ValidationError variant. Each is a Lambda-
// level protocol check (not a Core rejection) and lives in Lambda's
// namespace by convention.
pub const E_LAMBDA_SENDER_REDEEM_OWN_CHEQUE: &str = "E_LAMBDA_SENDER_REDEEM_OWN_CHEQUE";
pub const E_LAMBDA_RECEIVER_STATE_DRIFT: &str = "E_LAMBDA_RECEIVER_STATE_DRIFT";
pub const E_LAMBDA_ACK_WRONG_VALIDATOR: &str = "E_LAMBDA_ACK_WRONG_VALIDATOR";
pub const E_LAMBDA_ACK_INVALID_SIG: &str = "E_LAMBDA_ACK_INVALID_SIG";
pub const E_LAMBDA_ACK_NO_PENDING_FEE: &str = "E_LAMBDA_ACK_NO_PENDING_FEE";
pub const E_LAMBDA_ACK_FEE_MISMATCH: &str = "E_LAMBDA_ACK_FEE_MISMATCH";
pub const E_LAMBDA_FEE_INVALID_REDEEMER_SIG: &str = "E_LAMBDA_FEE_INVALID_REDEEMER_SIG";
pub const E_LAMBDA_FEE_MISSING_REDEEMER_SIG: &str = "E_LAMBDA_FEE_MISSING_REDEEMER_SIG";
pub const E_LAMBDA_FEE_UNKNOWN_TXID: &str = "E_LAMBDA_FEE_UNKNOWN_TXID";
pub const E_LAMBDA_FEE_SENDER_PK_MISMATCH: &str = "E_LAMBDA_FEE_SENDER_PK_MISMATCH";
pub const E_LAMBDA_FEE_NO_STORED_RECEIPT: &str = "E_LAMBDA_FEE_NO_STORED_RECEIPT";
pub const E_LAMBDA_FEE_ORIGINAL_WITNESS_CONFLICT: &str = "E_LAMBDA_FEE_ORIGINAL_WITNESS_CONFLICT";
pub const E_LAMBDA_FEE_TXID_MISMATCH: &str = "E_LAMBDA_FEE_TXID_MISMATCH";
pub const E_LAMBDA_FEE_VALIDATOR_PK_MISMATCH: &str = "E_LAMBDA_FEE_VALIDATOR_PK_MISMATCH";
pub const E_LAMBDA_FEE_NOT_RECIPIENT: &str = "E_LAMBDA_FEE_NOT_RECIPIENT";
// YP §19.6 amendment — per-slot fee verification (receiver-pays-only model)
pub const E_LAMBDA_FEE_SLOT_MISMATCH: &str = "E_LAMBDA_FEE_SLOT_MISMATCH";
pub const E_LAMBDA_FEE_SLOT_MISSING:  &str = "E_LAMBDA_FEE_SLOT_MISSING";

pub const E_LAMBDA_CL5_PROOF_INVALID: &str = "E_LAMBDA_CL5_PROOF_INVALID";
pub const E_LAMBDA_CL5_PROOF_MALFORMED: &str = "E_LAMBDA_CL5_PROOF_MALFORMED";
/// §15: redeem submitted without a CL5 execution proof. Mirror of CL1's
/// `"CL1: missing execution proof"` at consensus.rs:2428 — the pre-§15
/// "legacy mode (signature-only)" fallback is gone.
pub const E_LAMBDA_CL5_PROOF_MISSING: &str = "E_LAMBDA_CL5_PROOF_MISSING";
/// YPX-009 §7.2 (RULED 2026-09-10, the owner: "if pulse fail, reject"): Core
/// reported `audit_failed` on this execution — the validator's own Argon2id→
/// BLAKE3 audit chain and its transaction DB disagree — so the validator
/// REFUSES to sign this execution. Not the client's fault: retry elsewhere.
pub const E_LAMBDA_PULSE_AUDIT_FAILED: &str = "E_LAMBDA_PULSE_AUDIT_FAILED";
/// §5.2.2e part iii (KI#142): the ISSUER replayed the candidacy proof's
/// Fiat-Shamir sample from (key, attested tick) and it does not reproduce
/// `audit_hash` — the work was not done for this key at this round (or the
/// proof asks for more replay than the issuer allows). A hard refusal.
pub const E_LAMBDA_CANDIDACY_PULSE_WORK: &str = "E_LAMBDA_CANDIDACY_PULSE_WORK";
/// §5.2.2e part iii: one provisional request per candidate key per
/// `pulse_candidacy_min_request_interval_ticks` at this issuer. Wait.
pub const E_LAMBDA_CANDIDACY_PULSE_RATE: &str = "E_LAMBDA_CANDIDACY_PULSE_RATE";
/// Fable review 2026-10-01 F-3: the request carries a Nabla OODS attestation
/// whose tick is more than `ATTESTED_TICK_FUTURE_SKEW_SECS` (300 s,
/// lambda/protocol_lambda.toml; owner ruling 2026-10-02) ahead of the
/// validator's own wall clock — the TARDIS forward-only rule (YP §26.7.4,
/// YPX-003 §1.3.4), applied by the validator to a carried attestation. Refused before
/// any Core execution. No past-side bound exists.
pub const E_LAMBDA_ATTESTED_TICK_FUTURE: &str = "E_LAMBDA_ATTESTED_TICK_FUTURE";
pub const E_ATTESTATION_TXID_MISMATCH: &str = "E_ATTESTATION_TXID_MISMATCH";

// ── Nabla-level errors ─────────────────────────────────────────────────────
pub const E_NABLA_INVALID_DEED_PAYMENT: &str = "E_NABLA_INVALID_DEED_PAYMENT";
pub const E_NABLA_INVALID_DEED_DESTINATION: &str = "E_NABLA_INVALID_DEED_DESTINATION";
pub const E_NABLA_INVALID_RECEIPT: &str = "E_NABLA_INVALID_RECEIPT";
pub const E_NABLA_STATE_MISMATCH: &str = "E_NABLA_STATE_MISMATCH";
pub const E_NABLA_WALLET_BANNED: &str = "E_NABLA_WALLET_BANNED";
pub const E_NABLA_DOUBLE_SPEND_DETECTED: &str = "E_NABLA_DOUBLE_SPEND_DETECTED";
pub const E_NABLA_SMT_ERROR: &str = "E_NABLA_SMT_ERROR";
pub const E_NABLA_WAL_ERROR: &str = "E_NABLA_WAL_ERROR";
pub const E_NABLA_WAL_CORRUPTION: &str = "E_NABLA_WAL_CORRUPTION";
pub const E_NABLA_NBC_MISSING: &str = "E_NABLA_NBC_MISSING";
pub const E_NABLA_NBC_EXPIRED: &str = "E_NABLA_NBC_EXPIRED";
pub const E_NABLA_NBC_IDENTITY_MISMATCH: &str = "E_NABLA_NBC_IDENTITY_MISMATCH";
pub const E_NABLA_NBC_MALFORMED: &str = "E_NABLA_NBC_MALFORMED";
pub const E_NABLA_NBC_SIG_INVALID: &str = "E_NABLA_NBC_SIG_INVALID";
pub const E_NABLA_NBC_ISSUER_NOT_ROOT: &str = "E_NABLA_NBC_ISSUER_NOT_ROOT";
/// ForkSettlement §2.3 [R17] / [R‑MEDIUM-3] — register door step 5b′ refused
/// the registration's carried leg (`Registration::preimage`): it does not
/// reproduce the k-signed `commitment_hash` / `tx_hash`, disagrees with the
/// message's `old_state` / `client_pk` / seq, or the receipt lacks
/// ≥ max(k_tier, 3) distinct valid `receipt_commitment_sig`s. Wire form
/// `E_NABLA_LEG_UNVERIFIABLE|reason=<binding>|<human>`. Permanent for that
/// register (re-sending the same bytes fails identically).
pub const E_NABLA_LEG_UNVERIFIABLE: &str = "E_NABLA_LEG_UNVERIFIABLE";
/// KI#224 (owner ruling 2026-10-02) — register door step 5b⁗ refused the
/// registration because a witness key in its k-signed receipt is not the
/// subject of an R42 directory entry at THAT node (the witness sigs must walk
/// back to the root). Wire form `E_NABLA_WITNESS_NOT_IN_DIRECTORY|key=<hex>|
/// <human>`. RETRYABLE / per-node: a node whose directory is still filling
/// refuses an honest head another node accepts — the SDK walk tries the next
/// Nabla without counting a hard failure (like `E_POOL_CAP_PER_NABLA`).
pub const E_NABLA_WITNESS_NOT_IN_DIRECTORY: &str = "E_NABLA_WITNESS_NOT_IN_DIRECTORY";

// Phase A genesis-claim pool refusals — Session 13 monetary-expansion
// fix. Distinct codes so the SDK can route on the reason:
//   E_POOL_CAP_PER_NABLA — this Nabla's cycle counter is saturated;
//     the client should retry on a different Nabla. Transient.
//   E_POOL_CAP_MESH     — the mesh-wide cycle counter is saturated;
//     no Nabla can grant until cycle reset. Transient.
//   E_POOL_EXHAUSTED    — the pool is permanently drained (Airdrop
//     hits its 600,000 AXC cap, DevTreasury its 1M dev-AXC cap).
pub const E_POOL_CAP_PER_NABLA: &str = "E_POOL_CAP_PER_NABLA";
pub const E_POOL_CAP_MESH: &str = "E_POOL_CAP_MESH";
pub const E_POOL_EXHAUSTED: &str = "E_POOL_EXHAUSTED";

// Phase 2c — HTTP ingress validation errors. These fire on
// client-facing endpoints (/register, /clara, /query, /query-txid,
// /bridge) when the request body is malformed or the caller is on
// the wrong endpoint.
pub const E_NABLA_INVALID_JSON_BODY: &str = "E_NABLA_INVALID_JSON_BODY";
pub const E_NABLA_MISSING_FIELD: &str = "E_NABLA_MISSING_FIELD";
pub const E_NABLA_INVALID_HEX: &str = "E_NABLA_INVALID_HEX";
pub const E_NABLA_READER_REDIRECT: &str = "E_NABLA_READER_REDIRECT";
pub const E_NABLA_NOT_FOUND: &str = "E_NABLA_NOT_FOUND";
pub const E_NABLA_CLARA_ALREADY_REGISTERED: &str = "E_NABLA_CLARA_ALREADY_REGISTERED";
pub const E_NABLA_CLARA_INVALID_ATTESTATION: &str = "E_NABLA_CLARA_INVALID_ATTESTATION";
pub const E_NABLA_BRIDGE_PEER_UNREACHABLE: &str = "E_NABLA_BRIDGE_PEER_UNREACHABLE";
/// TCP send to a peer failed. Internal/operational. Logged at WARN by
/// the recv_loop and tick_loop drain paths so half-closed connection
/// loss is visible at default log levels — the previous debug-level
/// logging was the camouflage that hid the gossip-partition bug fixed
/// by `af79958`. Counter is also exposed on `/status` per peer.
pub const E_NABLA_TRANSPORT_SEND_FAILED: &str = "E_NABLA_TRANSPORT_SEND_FAILED";
/// Soak/test harness: gossip propagation invariant violation. Emitted
/// when a freshly-registered test wallet does not propagate to ≥N of
/// the 10 Nabla nodes within the convergence window. Catches gossip
/// transport bugs (zombie connections, mesh splits, fanout misconfig)
/// that the protocol's correctness defenses would otherwise mask as
/// chronic latency.
pub const E_SOAK_GOSSIP_PROPAGATION_FAILURE: &str = "E_SOAK_GOSSIP_PROPAGATION_FAILURE";

// ── CLARA registration (Nabla side) ────────────────────────────────────────
pub const E_CLARA_BAD_REQUEST: &str = "E_CLARA_BAD_REQUEST";
pub const E_CLARA_NOT_SELF_SEND: &str = "E_CLARA_NOT_SELF_SEND";
pub const E_CLARA_INSUFFICIENT_SIGNATURES: &str = "E_CLARA_INSUFFICIENT_SIGNATURES";
pub const E_CLARA_INCONSISTENT_BUNDLE: &str = "E_CLARA_INCONSISTENT_BUNDLE";
pub const E_CLARA_INVALID_VALIDATOR_SIGNATURE: &str = "E_CLARA_INVALID_VALIDATOR_SIGNATURE";
pub const E_CLARA_CONSUMED_ALREADY_TXID_REGISTERED: &str = "E_CLARA_CONSUMED_ALREADY_TXID_REGISTERED";
pub const E_CLARA_CONSUMED_ALREADY_GARBAGE: &str = "E_CLARA_CONSUMED_ALREADY_GARBAGE";
pub const E_CLARA_HEAL_ALREADY_REGISTERED: &str = "E_CLARA_HEAL_ALREADY_REGISTERED";
pub const E_CLARA_NOT_MARKED_HEAL: &str = "E_CLARA_NOT_MARKED_HEAL";
pub const E_CLARA_HEAL_TXID_MISMATCH: &str = "E_CLARA_HEAL_TXID_MISMATCH";
pub const E_CLARA_HEAL_TX_NOT_SELF_SEND: &str = "E_CLARA_HEAL_TX_NOT_SELF_SEND";
pub const E_CLARA_TOO_MANY_GARBAGE_STATES: &str = "E_CLARA_TOO_MANY_GARBAGE_STATES";
pub const E_CLARA_RATE_LIMITED: &str = "E_CLARA_RATE_LIMITED";
pub const E_CLARA_HEALED_BALANCE_MISMATCH: &str = "E_CLARA_HEALED_BALANCE_MISMATCH";


// ── ANTIE-level errors ─────────────────────────────────────────────────────
pub const E_ANTIE_MAILDIR_ERROR: &str = "E_ANTIE_MAILDIR_ERROR";
pub const E_ANTIE_EMAIL_PARSE_ERROR: &str = "E_ANTIE_EMAIL_PARSE_ERROR";
pub const E_ANTIE_INVALID_PAYLOAD: &str = "E_ANTIE_INVALID_PAYLOAD";
pub const E_ANTIE_CORE_TTL_EXPIRED: &str = "E_ANTIE_CORE_TTL_EXPIRED";
pub const E_ANTIE_RATE_LIMITED: &str = "E_ANTIE_RATE_LIMITED";
pub const E_ANTIE_VALIDATION_FAILED: &str = "E_ANTIE_VALIDATION_FAILED";
pub const E_ANTIE_LAMBDA_ERROR: &str = "E_ANTIE_LAMBDA_ERROR";
pub const E_ANTIE_CONFIG_ERROR: &str = "E_ANTIE_CONFIG_ERROR";
pub const E_ANTIE_SERIALIZATION: &str = "E_ANTIE_SERIALIZATION";
pub const E_ANTIE_IO_ERROR: &str = "E_ANTIE_IO_ERROR";
pub const E_ANTIE_UNAUTHORIZED: &str = "E_ANTIE_UNAUTHORIZED";
pub const E_ANTIE_NOT_FOUND: &str = "E_ANTIE_NOT_FOUND";
pub const E_ANTIE_CORE_ERROR: &str = "E_ANTIE_CORE_ERROR";
pub const E_ANTIE_WALLET_BANNED: &str = "E_ANTIE_WALLET_BANNED";
// KI#157 (2026-09-15) — structured twins of ANTIE-originated mail rejections.
pub const E_ANTIE_VALIDATOR_BUSY: &str = "E_ANTIE_VALIDATOR_BUSY";
pub const E_ANTIE_UNKNOWN_MESSAGE_TYPE: &str = "E_ANTIE_UNKNOWN_MESSAGE_TYPE";
pub const E_ANTIE_GENESIS_FAILED: &str = "E_ANTIE_GENESIS_FAILED";
pub const E_ANTIE_VBC_SIGN_FAILED: &str = "E_ANTIE_VBC_SIGN_FAILED";
pub const E_ANTIE_VBC_SIGN_COMMIT_FAILED: &str = "E_ANTIE_VBC_SIGN_COMMIT_FAILED";
pub const E_ANTIE_SET_AUTH_HASH_FAILED: &str = "E_ANTIE_SET_AUTH_HASH_FAILED";

// ── AVM / §23.14 ───────────────────────────────────────────────────────────
pub const E_AVM_LOAD_ERROR: &str = "E_AVM_LOAD_ERROR";
pub const E_AVM_EXECUTION_ERROR: &str = "E_AVM_EXECUTION_ERROR";
pub const E_AVM_RUNTIME_VERIFICATION_FAILED: &str = "E_AVM_RUNTIME_VERIFICATION_FAILED";
pub const E_AVM_AUDIT_TIMEOUT: &str = "E_AVM_AUDIT_TIMEOUT";
pub const E_AVM_VALIDATOR_BANNED: &str = "E_AVM_VALIDATOR_BANNED";
pub const E_AVM_PULSE_NOT_READY: &str = "E_AVM_PULSE_NOT_READY";

// ── Request envelope (§12) ─────────────────────────────────────────────────
pub const E_REQUEST_ENVELOPE_MALFORMED: &str = "E_REQUEST_ENVELOPE_MALFORMED";
pub const E_REQUEST_ENVELOPE_CLOCK_SKEW: &str = "E_REQUEST_ENVELOPE_CLOCK_SKEW";
pub const E_REQUEST_ENVELOPE_REPLAY: &str = "E_REQUEST_ENVELOPE_REPLAY";
pub const E_REQUEST_ENVELOPE_INVALID_SIG: &str = "E_REQUEST_ENVELOPE_INVALID_SIG";
pub const E_REQUEST_ENVELOPE_UNAUTHORIZED: &str = "E_REQUEST_ENVELOPE_UNAUTHORIZED";
