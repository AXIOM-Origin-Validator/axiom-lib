//! Recovery hints — the 8-variant enum that maps 1:1 to SDK recovery actions.
//! See `AXIOM_YellowPaper_Errors.md` §5.

use serde::{Deserialize, Serialize};

/// Recovery strategy hint. Present on `ErrorResponse` when the
/// category is `RecoverableDrift` or `Operational`. The SDK maps each
/// variant 1:1 to a specific method on the `Wallet` object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum RecoveryHint {
    /// Run the CLARA heal flow on the next normal send — `wallet.heal()`
    /// re-anchors the wallet via a key-proved, validator-witnessed
    /// self-send. The sanctioned recovery for wallet/validator state
    /// divergence: CLAUDE.md §14 — clients never resync from Nabla; heal
    /// is the only recovery path.
    ///
    /// Used for: E_SABR_HASH_MISMATCH, S-ABR state-chain mismatch,
    /// E_INVALID_WALLET_SEQ, receiver state drift, burn-path drift.
    ClaraHealNextSend = 1,

    /// Retry the SAME request with the SAME payload on the SAME
    /// validator. YPX-016 witness response cache will return the
    /// cached response if the validator already committed but the
    /// response was lost in transit. Byte-identical retries are safe.
    ///
    /// Used for: transient timeout on an overlap validator, network
    /// flakes during witness collection.
    RetrySameValidator = 2,

    /// Try a different validator from the backup pool. The current
    /// one is unreachable or has an invalid credential. The SDK's
    /// select_validators should return a different k=3 set on retry.
    ///
    /// Used for: E_VBC_EXPIRED, E_VBC_NOT_YET_VALID, E_VBC_ROOT_KEY_MISMATCH
    /// (all client-visible as "this validator is unusable"),
    /// E_LAMBDA_CONSENSUS_TIMEOUT.
    RetryDifferentValidator = 3,

    /// Dedup the cheque list by (txid, validator_id) per YP §17.9.4.0
    /// before retrying the redeem. The bundle submitted had duplicate
    /// entries from the same validator.
    ///
    /// Used for: E_CHEQUE_INCONSISTENT_BUNDLE, E_INSUFFICIENT_CHEQUES
    /// when duplicates are present.
    DedupChequeBundle = 4,

    /// Wait for the duration specified in `retry_after_secs` (top-level
    /// ErrorResponse field), then retry. Network is healthy but the
    /// operation can't proceed yet (rate limit, maturity delay, cooldown).
    ///
    /// Used for: E_LAMBDA_RATE_LIMIT_EXCEEDED, E_CLARA_RATE_LIMITED,
    /// E_ANTIE_RATE_LIMITED, E_ORACLE_MATURITY_NOT_REACHED,
    /// E_ORACLE_CLAIM_TOO_SOON.
    WaitAndRetry = 5,

    /// File the S6 ban challenge flow: collect ≥k-1 endorsements from
    /// other validators attesting the ban was wrong, file via
    /// `nabla_client.challenge_ban()`, retry the original request
    /// after the challenge window elapses.
    ///
    /// Used for: E_NABLA_WALLET_BANNED only. NOT used for
    /// E_GENESIS_STAKE_LOCKED (which has no recovery — 3-year wait).
    S6BanChallenge = 6,

    /// File a FACT chain compression checkpoint to reduce chain depth,
    /// then retry. Chain has exceeded MAX_FACT_DEPTH.
    ///
    /// Used for: E_FACT_CHAIN_TOO_DEEP.
    FactChainCompress = 7,

    /// Clear local FACT scars via the burn path before retrying the
    /// original operation. Required for Ark unload and certain other
    /// state-sensitive operations.
    ///
    /// Used for: E_ARK_UNLOAD_SCARRED, E_TOO_MANY_UNRESOLVED_SCARS.
    BurnExistingScars = 8,
}
