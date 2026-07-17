//! Error categories. See `AXIOM_YellowPaper_Errors.md` §2.3.

use serde::{Deserialize, Serialize};

/// Client dispatch category. Every `ErrorResponse` has exactly one.
///
/// The client's behavior is fully determined by this field plus
/// `recovery` (for `RecoverableDrift` and `Operational`). See the
/// per-category behavior contract in §3 of the Errors Yellow Paper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum ErrorCategory {
    /// The network refused the operation for a cryptographic, consensus,
    /// or policy reason. The request is WRONG in a way the client
    /// cannot transparently fix.
    ///
    /// Client contract: MUST NOT retry blindly. MUST surface to user
    /// with actionable guidance. MAY offer code-specific recovery UI.
    ProtocolReject = 0,

    /// The request failed because local state or payload drifted from
    /// network reality. A well-known recovery flow exists.
    ///
    /// Client contract: MUST dispatch to the `recovery` hint and retry
    /// once. On second failure, escalate.
    RecoverableDrift = 1,

    /// Transient condition. Client SHOULD retry with whatever backoff
    /// strategy its deployment context prefers.
    ///
    /// Client contract: respect `retry_after_secs` if present. Otherwise
    /// pick client-side policy. Retry policy is NOT mandated by the
    /// protocol.
    Operational = 2,

    /// The client sent a malformed or invariant-violating request.
    /// Bug is in the client implementation.
    ///
    /// Client contract: MUST NOT retry. Log with full context. Panic
    /// in debug. Generic "client issue" to users in production.
    ClientBug = 3,

    /// Internal failure inside Core, Lambda, Nabla, or ANTIE. Not the
    /// client's fault.
    ///
    /// Client contract: MAY retry once on a different node. Log for
    /// operator investigation. Neutral "server issue" to users.
    Internal = 4,
}

impl ErrorCategory {
    /// Is retrying safe for this category?
    pub fn is_retryable(self) -> bool {
        matches!(
            self,
            Self::RecoverableDrift | Self::Operational | Self::Internal
        )
    }

    /// Should this error be surfaced to the end-user?
    /// (ClientBug and Internal should not leak details to users.)
    pub fn is_user_visible(self) -> bool {
        matches!(self, Self::ProtocolReject | Self::Operational)
    }

    /// One-word label for logs and UI.
    pub const fn label(self) -> &'static str {
        match self {
            Self::ProtocolReject => "rejected",
            Self::RecoverableDrift => "drift",
            Self::Operational => "transient",
            Self::ClientBug => "client-bug",
            Self::Internal => "internal",
        }
    }
}
