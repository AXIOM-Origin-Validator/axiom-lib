//! The `ErrorResponse` wire format. See `AXIOM_YellowPaper_Errors.md` §2.

use alloc::string::String;
use serde::{Deserialize, Serialize};

use crate::{ErrorCategory, ErrorCode, ErrorDetail, RecoveryHint, ERROR_RESPONSE_VERSION};

/// The canonical error response returned by Core, Lambda, Nabla, and
/// ANTIE when any operation fails.
///
/// See the class-level docs on each field for the stability contract.
/// Serialization format is CBOR; JSON is a debugging convenience only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Stable code identifying the error class. Clients MUST match
    /// on this for dispatch. MUST NOT depend on `message`.
    pub code: ErrorCode,

    /// What the client is required to do with this error.
    pub category: ErrorCategory,

    /// Human-readable message, English. Never localized (localization
    /// is a client concern keyed on `code`). Stable enough for logs
    /// and grep; client logic MUST NOT depend on the exact string.
    pub message: String,

    /// Optional structured context. Fields depend on `code`. Some
    /// fields are debug-gated per §7 of the Errors Yellow Paper.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<ErrorDetail>,

    /// Recovery hint. Present only for `RecoverableDrift` and
    /// `Operational` categories.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery: Option<RecoveryHint>,

    /// Seconds to wait before retrying. Present only for `Operational`
    /// errors with a definite retry window. If absent on an
    /// Operational error, the client picks its own backoff policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after_secs: Option<u32>,

    /// Yellow Paper section reference. SDK error display SHOULD
    /// include this as a clickable link. Examples: "§17.9.4.0",
    /// "YPX-018 §2.3".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yp_reference: Option<String>,

    /// Request correlation ID. Echoes the client's `request_id` if
    /// provided. Used for log tracing across layers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,

    /// Wire format version. Starts at 1. Clients MUST check this
    /// field first and fall back to legacy string parsing if they
    /// don't understand the version.
    pub version: u8,
}

impl ErrorResponse {
    /// Construct a minimal `ErrorResponse` with just the three
    /// mandatory fields. The `code` argument accepts either an
    /// `ErrorCode` directly or a `&'static str` constant from
    /// `error_code` (which is preferred for call sites).
    pub fn new(
        code: impl Into<ErrorCode>,
        category: ErrorCategory,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            category,
            message: message.into(),
            detail: None,
            recovery: None,
            retry_after_secs: None,
            yp_reference: None,
            request_id: None,
            version: ERROR_RESPONSE_VERSION,
        }
    }

    /// Builder: attach a typed detail.
    pub fn with_detail(mut self, detail: ErrorDetail) -> Self {
        self.detail = Some(detail);
        self
    }

    /// Builder: attach a recovery hint. Panics in debug if the
    /// category is not `RecoverableDrift` or `Operational` — those
    /// are the only categories where a recovery hint is valid per
    /// the spec.
    pub fn with_recovery(mut self, hint: RecoveryHint) -> Self {
        debug_assert!(
            matches!(
                self.category,
                ErrorCategory::RecoverableDrift | ErrorCategory::Operational
            ),
            "recovery hint only valid for RecoverableDrift/Operational categories, got {:?}",
            self.category
        );
        self.recovery = Some(hint);
        self
    }

    /// Builder: set the retry_after seconds. Only meaningful for
    /// Operational errors.
    pub fn with_retry_after(mut self, secs: u32) -> Self {
        self.retry_after_secs = Some(secs);
        self
    }

    /// Builder: attach a Yellow Paper section reference.
    pub fn with_yp_reference(mut self, reference: impl Into<String>) -> Self {
        self.yp_reference = Some(reference.into());
        self
    }

    /// Builder: set the correlation request ID.
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    /// Shorthand: is the category retryable?
    pub fn is_retryable(&self) -> bool {
        self.category.is_retryable()
    }

    /// Shorthand: should this be shown to the user?
    pub fn is_user_visible(&self) -> bool {
        self.category.is_user_visible()
    }

    /// Return a copy of this ErrorResponse with DEBUG-gated fields in
    /// `detail` stripped. Used at untrusted boundaries per §7 of the
    /// Errors Yellow Paper — when the error is being emitted to a
    /// caller who is NOT the wallet owner (e.g., a peer validator
    /// querying on behalf of someone else), the validator's internal
    /// state view (current balance, stored state_id, stored wallet_seq)
    /// must not leak.
    ///
    /// The rule is "strip, don't remove": the fields become `None`
    /// rather than the whole `detail` being dropped, so the client
    /// still learns the error class and knows what recovery to
    /// attempt. Only the validator's own view is redacted.
    ///
    /// Call this at the HTTP response boundary when `RequestEnvelope`
    /// authentication did NOT verify the caller as the wallet owner.
    /// The default Lambda flow — where the client signs the witness
    /// request with their own wallet key — does NOT need redaction,
    /// because the recipient IS the wallet owner.
    pub fn redact_debug_fields(mut self) -> Self {
        use crate::error_detail::ErrorDetail;
        self.detail = self.detail.map(|d| match d {
            ErrorDetail::StateChainMismatch(mut scm) => {
                scm.validator_stored_state_id = None;
                scm.validator_wallet_seq = None;
                ErrorDetail::StateChainMismatch(scm)
            }
            ErrorDetail::Balance(mut bd) => {
                bd.current_balance = None;
                ErrorDetail::Balance(bd)
            }
            // Other detail variants have no DEBUG-gated fields —
            // pass through unchanged. If a future variant adds gated
            // fields, add a case here.
            other => other,
        });
        self
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error_code;
    use crate::error_detail::StateChainMismatchDetail;
    use alloc::vec::Vec;

    #[test]
    fn minimal_error_response_roundtrip() {
        let err = ErrorResponse::new(
            error_code::E_INSUFFICIENT_BALANCE,
            ErrorCategory::ProtocolReject,
            "Wallet has insufficient balance",
        );
        let mut buf = Vec::new();
        ciborium::into_writer(&err, &mut buf).unwrap();
        let decoded: ErrorResponse = ciborium::from_reader(buf.as_slice()).unwrap();
        assert_eq!(err, decoded);
        assert_eq!(decoded.version, ERROR_RESPONSE_VERSION);
    }

    #[test]
    fn hash_mismatch_with_detail_and_recovery() {
        let detail = ErrorDetail::StateChainMismatch(StateChainMismatchDetail {
            requested_consumed_state_id: [0x12; 32],
            requested_wallet_seq: 42,
            validator_stored_state_id: Some([0xab; 32]),
            validator_wallet_seq: Some(44),
        });
        let err = ErrorResponse::new(
            error_code::E_SABR_HASH_MISMATCH,
            ErrorCategory::RecoverableDrift,
            "Wallet state does not match validator's stored state",
        )
        .with_detail(detail)
        .with_recovery(RecoveryHint::ClaraHealNextSend)
        .with_yp_reference("§17.10.14 CLARA + YPX-018")
        .with_request_id("pmc-a1b2c3d4-v3");

        let mut buf = Vec::new();
        ciborium::into_writer(&err, &mut buf).unwrap();
        let decoded: ErrorResponse = ciborium::from_reader(buf.as_slice()).unwrap();
        assert_eq!(err, decoded);
        assert!(decoded.is_retryable());
        assert!(!decoded.is_user_visible());
    }

    #[test]
    fn operational_with_retry_after() {
        let err = ErrorResponse::new(
            error_code::E_LAMBDA_RATE_LIMIT_EXCEEDED,
            ErrorCategory::Operational,
            "Validator is busy",
        )
        .with_recovery(RecoveryHint::WaitAndRetry)
        .with_retry_after(15);
        let mut buf = Vec::new();
        ciborium::into_writer(&err, &mut buf).unwrap();
        let decoded: ErrorResponse = ciborium::from_reader(buf.as_slice()).unwrap();
        assert_eq!(err, decoded);
        assert_eq!(decoded.retry_after_secs, Some(15));
    }

    #[test]
    fn client_bug_no_retry_no_recovery() {
        let err = ErrorResponse::new(
            error_code::E_INVALID_CLIENT_SIG,
            ErrorCategory::ClientBug,
            "Client signature verification failed",
        );
        assert!(!err.is_retryable());
        assert!(!err.is_user_visible());
        assert!(err.recovery.is_none());
    }

    #[test]
    fn redact_debug_fields_strips_balance_current() {
        use crate::error_detail::BalanceDetail;
        let err = ErrorResponse::new(
            error_code::E_INSUFFICIENT_BALANCE,
            ErrorCategory::ProtocolReject,
            "not enough",
        )
        .with_detail(ErrorDetail::Balance(BalanceDetail {
            requested_amount: 1_000_000,
            current_balance: Some(500_000),
        }));

        let redacted = err.clone().redact_debug_fields();
        match redacted.detail {
            Some(ErrorDetail::Balance(bd)) => {
                assert_eq!(bd.requested_amount, 1_000_000);
                assert_eq!(bd.current_balance, None, "current_balance must be stripped");
            }
            _ => panic!("expected Balance detail"),
        }

        // Original is untouched — redact_debug_fields takes self by value.
        match err.detail {
            Some(ErrorDetail::Balance(bd)) => {
                assert_eq!(bd.current_balance, Some(500_000));
            }
            _ => panic!("expected Balance detail"),
        }
    }

    #[test]
    fn redact_debug_fields_strips_state_chain_stored_view() {
        let err = ErrorResponse::new(
            error_code::E_SABR_HASH_MISMATCH,
            ErrorCategory::RecoverableDrift,
            "drift",
        )
        .with_detail(ErrorDetail::StateChainMismatch(StateChainMismatchDetail {
            requested_consumed_state_id: [0x11; 32],
            requested_wallet_seq: 7,
            validator_stored_state_id: Some([0x22; 32]),
            validator_wallet_seq: Some(9),
        }))
        .with_recovery(RecoveryHint::ClaraHealNextSend);

        let redacted = err.redact_debug_fields();
        match redacted.detail {
            Some(ErrorDetail::StateChainMismatch(scm)) => {
                assert_eq!(scm.requested_consumed_state_id, [0x11; 32]);
                assert_eq!(scm.requested_wallet_seq, 7);
                assert_eq!(scm.validator_stored_state_id, None);
                assert_eq!(scm.validator_wallet_seq, None);
            }
            _ => panic!("expected StateChainMismatch detail"),
        }
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "recovery hint only valid")]
    fn with_recovery_panics_on_protocol_reject() {
        ErrorResponse::new(
            error_code::E_INSUFFICIENT_BALANCE,
            ErrorCategory::ProtocolReject,
            "nope",
        )
        .with_recovery(RecoveryHint::ClaraHealNextSend);
    }
}
