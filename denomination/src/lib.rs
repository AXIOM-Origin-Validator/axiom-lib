//! AXIOM Denomination Library
//!
//! Converts between the three layers of AXIOM currency:
//!
//! | Layer    | Name    | Used by              | Example              |
//! |----------|---------|----------------------|----------------------|
//! | Protocol | atoms   | Core (code only)     | 5,000,000,000 atoms  |
//! | Market   | AXC     | Exchanges, specs     | 0.5 AXC              |
//! | User     | L$      | Wallet UI, invoices  | 500 L$ (at dv=3)     |
//!
//! # Constants
//!
//! - 1 AXC = 10^10 atoms (10,000,000,000)
//! - 1 AXC = 10^N L$ where N = digit_version
//! - Total supply: 100,000,000 AXC = 10^18 atoms
//!
//! # Usage
//!
//! ```rust
//! use axiom_denomination::*;
//!
//! let atoms = axc(1);             // 10,000,000,000 atoms
//! let atoms = axc_f(0.5);         // 5,000,000,000 atoms
//! let val = to_axc(atoms);        // 0.5
//! let s = format_axc(atoms);      // "0.5 AXC"
//! let s = format_ldollar(atoms, 3); // "500 L$"
//! ```
//!
//! # Digit Version
//!
//! The default digit_version is set in Cargo.toml under `[package.metadata.axiom]`.
//! At runtime, update it from the network via `set_digit_version()`.
//! The Console (§7.7) manages digit_version for the network.

// no_std unless the consumer enables the `std` feature. This means a
// consumer that takes the crate with default-features = false (e.g.
// the AVM-guest path via core-logic) automatically gets the no_std
// build — no need for an explicit `features = ["no_std"]` toggle.
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};
#[cfg(not(feature = "std"))]
use alloc::format;

// ============================================================================
// Constants
// ============================================================================

// `ATOMS_PER_AXC` comes from `protocol_denomination.toml` via build.rs. (The dust floor is Core policy — axiom_core_logic::validation::MINIMUM_TX_ATOMS — not here.)
// This is the workspace's single source of truth for the atom unit — every crate
// (Core, Lambda, Nabla, SDK, webclient, tools) and every non-Rust artifact
// (HTML dashboards, Swift Mac wallet) must pull from here, never a
// hand-copied literal. Drift between sites is the bug class this codegen
// closes; the failure mode would be off-by-one balances at the cross-
// language boundary, exactly the place where a hand-copied literal silently
// goes stale.
include!(concat!(env!("OUT_DIR"), "/denomination_constants.rs"));

/// Default digit_version (genesis = 0, meaning 1 AXC = 1 L$).
/// Developers can change this default. At runtime, use set_digit_version().
pub const DEFAULT_DIGIT_VERSION: u32 = 0;

// ============================================================================
// 乖乖 — single source of truth for the decorative boot canary.
// ============================================================================
//
// Lives here so every Rust artifact in the workspace that depends on
// axiom-denomination (every binary, the WASM webclient, the Core ELF)
// gets the art baked in by `include_str!`. If the bytes of the art
// file change, every dependent crate is recompiled and the change
// cascades through every produced binary — including a new CoreID.
// Art absent from a built artifact ⇒ that artifact does not depend
// on this crate ⇒ unit-conversion is not consolidated.
//
// Cosmetic rules (non-negotiable, do not "fix"): the flavor stays
// 椰子 (coconut); dismissal in any UI overlay is click-anywhere,
// never a button labeled "open" / "打開".

/// The 乖乖 ASCII art. Single source of truth in the workspace.
/// Embedded at compile time from `denomination/assets/kuaikuai.txt`.
pub const KUAIKUAI_ART: &str = include_str!("../assets/kuaikuai.txt");

/// Force `KUAIKUAI_ART` into every artifact that links this crate, even
/// if no code path reads it (the AVM-guest ELF in particular). Without
/// `#[used]` the linker would drop the bytes as dead data.
#[used]
static KUAIKUAI_ART_ANCHOR: &[u8] = KUAIKUAI_ART.as_bytes();

/// Print the boot charm to stderr when running under an interactive
/// TTY, or when `AXIOM_KUAIKUAI=1` forces it. No-op otherwise — designed
/// so systemd/journald never receive the art.
///
/// `node` is the operator-side label this binary identifies as —
/// `"lambda"`, `"antie"`, `"alpha"`. Not signed, not persisted; status
/// line only.
///
/// Must be called BEFORE tracing/log subscriber init or the art can
/// interleave with structured log lines.
///
/// Decorative only — failures are silently swallowed; the boot path
/// is never blocked by the canary.
#[cfg(feature = "std")]
pub fn print_if_tty(node: &str) {
    use std::io::{IsTerminal, Write};

    let stderr = std::io::stderr();
    let forced = std::env::var("AXIOM_KUAIKUAI").ok().as_deref() == Some("1");
    if !stderr.is_terminal() && !forced {
        return;
    }

    let when = chrono::Local::now().format("%Y-%m-%d %H:%M:%S %Z");
    let mut handle = stderr.lock();
    let _ = handle.write_all(KUAIKUAI_ART.as_bytes());
    let _ = writeln!(handle);
    let _ = writeln!(
        handle,
        "乖乖 (椰子口味) placed · node {} · started {}",
        node, when,
    );
    let _ = writeln!(handle, "本機乖乖聽話 — 請勿打開，請勿食用");
    let _ = writeln!(handle);
}

// ============================================================================
// Runtime digit_version (thread-safe global)
// ============================================================================

#[cfg(feature = "std")]
use std::sync::atomic::{AtomicU32, Ordering};

#[cfg(feature = "std")]
static DIGIT_VERSION: AtomicU32 = AtomicU32::new(DEFAULT_DIGIT_VERSION);

/// Set the current digit_version (from Console/VSP network response).
#[cfg(feature = "std")]
pub fn set_digit_version(dv: u32) {
    DIGIT_VERSION.store(dv, Ordering::Relaxed);
}

/// Get the current digit_version.
#[cfg(feature = "std")]
pub fn digit_version() -> u32 {
    DIGIT_VERSION.load(Ordering::Relaxed)
}

// no_std: no global state, always pass digit_version explicitly
#[cfg(not(feature = "std"))]
pub fn digit_version() -> u32 {
    DEFAULT_DIGIT_VERSION
}

// ============================================================================
// AXC → atoms (for developers building transactions)
// ============================================================================

/// Convert whole AXC to atoms. Compile-time safe.
///
/// ```
/// use axiom_denomination::axc;
/// assert_eq!(axc(1), 10_000_000_000);
/// assert_eq!(axc(100), 1_000_000_000_000);
/// ```
pub const fn axc(whole_axc: u64) -> u64 {
    whole_axc * ATOMS_PER_AXC
}

/// Convert fractional AXC to atoms.
///
/// ```
/// use axiom_denomination::axc_f;
/// assert_eq!(axc_f(0.5), 5_000_000_000);
/// assert_eq!(axc_f(1.5), 15_000_000_000);
/// assert_eq!(axc_f(0.00005), 500_000);  // 0.00005 AXC
/// ```
pub fn axc_f(axc_amount: f64) -> u64 {
    (axc_amount * ATOMS_PER_AXC as f64) as u64
}

/// Convert L$ to atoms using a specific digit_version.
///
/// ```
/// use axiom_denomination::ldollar;
/// assert_eq!(ldollar(1000, 0), 10_000_000_000_000);  // 1000 L$ = 1000 AXC at dv=0
/// assert_eq!(ldollar(1000, 3), 10_000_000_000);       // 1000 L$ = 1 AXC at dv=3
/// ```
pub fn ldollar(ldollar_amount: u64, dv: u32) -> u64 {
    if dv == 0 {
        ldollar_amount * ATOMS_PER_AXC
    } else {
        let divisor = 10u64.pow(dv);
        (ldollar_amount * ATOMS_PER_AXC) / divisor
    }
}

/// Convert L$ (fractional) to atoms.
pub fn ldollar_f(ldollar_amount: f64, dv: u32) -> u64 {
    let axc = if dv == 0 {
        ldollar_amount
    } else {
        // 10u64.pow(dv) is exact and no_std-clean (vs f64::powi which
        // needs libm in no_std). dv is realistically 0..=18 in AXIOM.
        ldollar_amount / (10u64.pow(dv) as f64)
    };
    (axc * ATOMS_PER_AXC as f64) as u64
}

// ============================================================================
// atoms → AXC / L$ (for display and API responses)
// ============================================================================

/// Convert atoms to AXC (floating point).
pub fn to_axc(atoms: u64) -> f64 {
    atoms as f64 / ATOMS_PER_AXC as f64
}

/// Convert atoms to L$ (floating point) using a specific digit_version.
pub fn to_ldollar(atoms: u64, dv: u32) -> f64 {
    let axc = to_axc(atoms);
    if dv == 0 {
        axc
    } else {
        // 10u64.pow(dv) is exact and no_std-clean; dv is 0..=18 in AXIOM.
        axc * (10u64.pow(dv) as f64)
    }
}

/// Convert atoms to L$ using the current runtime digit_version.
pub fn to_ldollar_current(atoms: u64) -> f64 {
    to_ldollar(atoms, digit_version())
}

// ============================================================================
// Formatting (human-readable strings)
// ============================================================================

/// Format atoms as AXC string: "1.5 AXC"
pub fn format_axc(atoms: u64) -> String {
    let whole = atoms / ATOMS_PER_AXC;
    let frac = atoms % ATOMS_PER_AXC;
    if frac == 0 {
        format!("{} AXC", whole)
    } else {
        // Remove trailing zeros from fractional part
        let frac_str = format!("{:010}", frac);
        let trimmed = frac_str.trim_end_matches('0');
        format!("{}.{} AXC", whole, trimmed)
    }
}

/// Format atoms as L$ string with digit_version: "1,500 L$"
pub fn format_ldollar(atoms: u64, dv: u32) -> String {
    let value = to_ldollar(atoms, dv);
    if value >= 1.0 && value == (value as u64) as f64 {
        // Whole number — use comma formatting
        format!("{} L$", format_with_commas(value as u64))
    } else if value >= 0.01 {
        format!("{:.2} L$", value)
    } else {
        format!("{} L$", value)
    }
}

/// Format atoms as L$ using current runtime digit_version.
pub fn format_ldollar_current(atoms: u64) -> String {
    format_ldollar(atoms, digit_version())
}

/// Format atoms as L$ string with integer-math precision and adaptive
/// trailing-zero trimming. Mirrors `format_axc`'s shape rather than
/// the f64-based `format_ldollar` above.
///
/// Why a second L$ formatter: `format_ldollar` uses f64 division and
/// caps the fractional display at 2 digits via `"{:.2}"`. That's fine
/// for everyday "0.84 L$" reads but silently truncates sub-cent
/// precision — a wallet holding 8_420_000_000 atoms at d_v=0 (=
/// 0.842 L$ exactly, since 1 L$ = 1 AXC at d_v=0) renders as
/// "0.84 L$", and there's no way to tell whether the user is
/// rounding from 0.840 or 0.844. The integer-math version below
/// keeps every atom of precision visible while still trimming
/// trailing zeros down to a 2-decimal minimum so whole-cent values
/// stay tidy.
///
/// Rules:
///   - atoms_per_ldollar = 10^(10 - dv)
///   - whole = atoms / atoms_per_ldollar (comma-formatted)
///   - frac = atoms % atoms_per_ldollar, rendered with (10 - dv)
///     leading zeros, then trailing zeros trimmed down to a 2-
///     character floor.
///   - Whole-L$ values omit the fractional part entirely.
///
/// Examples at d_v=0 (1 L$ = 10^10 atoms = 1 AXC):
///   8_420_000_000  → "0.842 L$"
///   10_000_000_000 → "1 L$"
///   1              → "0.0000000001 L$"
///   100_000_000_000 → "10 L$"
///
/// Examples at d_v=2 (1 L$ = 10^8 atoms = 0.01 AXC):
///   100_000_000   → "1 L$"
///   8_420_000_000 → "84.20 L$" (84.2 whole, fractional 20 = "20" after trim)
///   wait actually 8_420_000_000 / 10^8 = 84, frac = 20_000_000
///   "{:08}" of 20_000_000 = "20000000", trim trailing 0s to floor 2 → "20"
///   So "84.20 L$" ✓
pub fn format_ldollar_precise(atoms: u64, dv: u32) -> String {
    let frac_digits = 10u32.saturating_sub(dv) as usize;
    if frac_digits == 0 {
        // Degenerate d_v ≥ 10 → 1 atom per L$. No fractional part.
        return format!("{} L$", format_with_commas(atoms));
    }
    let atoms_per_ldollar = 10u64.pow(frac_digits as u32);
    let whole = atoms / atoms_per_ldollar;
    let frac = atoms % atoms_per_ldollar;
    let whole_str = format_with_commas(whole);
    if frac == 0 {
        return format!("{} L$", whole_str);
    }
    // Pad the fractional part to its full atom-precision width, then
    // trim trailing zeros down to a 2-character floor (canonical
    // "cents" minimum).
    let frac_str = format!("{:0width$}", frac, width = frac_digits);
    let mut trimmed: String = frac_str;
    while trimmed.len() > 2 && trimmed.ends_with('0') {
        trimmed.pop();
    }
    format!("{}.{} L$", whole_str, trimmed)
}

/// Format atoms as precision L$ using current runtime digit_version.
pub fn format_ldollar_precise_current(atoms: u64) -> String {
    format_ldollar_precise(atoms, digit_version())
}

/// Money-style L$: at most 2 decimal places (like fiat). Integer math —
/// no float artifacts. Whole amounts drop the decimals ("1,000 L$"); a
/// non-zero amount smaller than 0.01 L$ renders as "< 0.01 L$" rather
/// than a misleading "0.00" (AXIOM L$ has finer granularity than fiat,
/// down to an atom — the exact value is always available in AXC).
pub fn format_ldollar_2dp(atoms: u64, dv: u32) -> String {
    let frac_digits = 10u32.saturating_sub(dv) as usize;
    if frac_digits == 0 {
        // d_v ≥ 10 → 1 atom per L$, no fractional part.
        return format!("{} L$", format_with_commas(atoms));
    }
    let per = 10u64.pow(frac_digits as u32); // atoms per 1 L$
    let whole = atoms / per;
    let frac = atoms % per; // sub-L$ atoms
    if frac == 0 {
        return format!("{} L$", format_with_commas(whole));
    }
    // First two decimal digits (truncated, not rounded — never overstate).
    let cents = if frac_digits >= 2 {
        frac / 10u64.pow((frac_digits - 2) as u32)
    } else {
        frac * 10u64.pow((2 - frac_digits) as u32)
    };
    if whole == 0 && cents == 0 {
        // Dust — smaller than the 2dp money view can show. Rather than a
        // vague "< 0.01 L$", show the REAL figure at finer precision
        // (truncated, never rounded up). Up to 4 decimal places; below
        // that granularity fall back to a precise floor marker. `frac` is
        // the sub-L$ atom remainder, `per` (= 10^frac_digits) atoms per L$.
        let dp = 4usize.min(frac_digits);
        let scaled = if frac_digits >= dp {
            frac / 10u64.pow((frac_digits - dp) as u32)
        } else {
            frac * 10u64.pow((dp - frac_digits) as u32)
        };
        if scaled == 0 {
            // Even at `dp` decimals this truncates to zero — genuinely
            // microscopic. Show the smallest representable floor (e.g.
            // "< 0.0001 L$"); the exact value is always in the AXC line.
            return format!("< 0.{:0>width$} L$", 1, width = dp);
        }
        // Render `dp` decimals, trimming trailing zeros down to a 3-place
        // floor so "0.0034" not "0.00340000", but enough places to read
        // as a sub-cent amount.
        let mut frac_str = format!("{:0width$}", scaled, width = dp);
        while frac_str.len() > 3 && frac_str.ends_with('0') {
            frac_str.pop();
        }
        return format!("0.{} L$", frac_str);
    }
    format!("{}.{:02} L$", format_with_commas(whole), cents)
}

pub fn format_ldollar_2dp_current(atoms: u64) -> String {
    format_ldollar_2dp(atoms, digit_version())
}

/// Format u64 with comma separators: 1000000 → "1,000,000"
fn format_with_commas(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

// ============================================================================
// Parsing (from user input / API requests)
// ============================================================================

/// Parse an AXC string to atoms: "1.5" → 15,000,000,000
pub fn parse_axc(s: &str) -> Result<u64, &'static str> {
    let s = s.trim().trim_end_matches(" AXC").trim_end_matches(" axc").trim();
    if let Some(dot_pos) = s.find('.') {
        let whole: u64 = s[..dot_pos].parse().map_err(|_| "invalid AXC amount")?;
        let frac_str = &s[dot_pos + 1..];
        if frac_str.len() > 10 {
            return Err("too many decimal places (max 10 for atoms precision)");
        }
        let padded = format!("{:0<10}", frac_str);
        let frac: u64 = padded.parse().map_err(|_| "invalid decimal")?;
        Ok(whole * ATOMS_PER_AXC + frac)
    } else {
        let whole: u64 = s.parse().map_err(|_| "invalid AXC amount")?;
        Ok(whole * ATOMS_PER_AXC)
    }
}

/// Parse an L$ string to atoms: "1,500" at dv=3 → 15,000,000,000
pub fn parse_ldollar(s: &str, dv: u32) -> Result<u64, &'static str> {
    let s = s.trim()
        .trim_end_matches(" L$")
        .trim_end_matches(" l$")
        .replace(',', "")
        .trim()
        .to_string();
    let value: f64 = s.parse().map_err(|_| "invalid L$ amount")?;
    Ok(ldollar_f(value, dv))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_axc_to_atoms() {
        assert_eq!(axc(0), 0);
        assert_eq!(axc(1), 10_000_000_000);
        assert_eq!(axc(100), 1_000_000_000_000);
        assert_eq!(axc(100_000_000), 1_000_000_000_000_000_000); // total supply in atoms
    }

    #[test]
    fn test_axc_f() {
        assert_eq!(axc_f(0.5), 5_000_000_000);
        assert_eq!(axc_f(1.5), 15_000_000_000);
        assert_eq!(axc_f(0.00005), 500_000); // dust limit
    }

    #[test]
    fn test_total_supply_fits_u64() {
        let total_atoms = axc(100_000_000);
        assert_eq!(total_atoms, 1_000_000_000_000_000_000); // 10^18
        assert!(total_atoms < u64::MAX); // fits in u64
    }

    #[test]
    fn test_to_axc() {
        assert_eq!(to_axc(10_000_000_000), 1.0);
        assert_eq!(to_axc(15_000_000_000), 1.5);
        assert_eq!(to_axc(500_000), 0.00005);
    }

    #[test]
    fn test_ldollar_dv0() {
        // dv=0: 1 AXC = 1 L$
        assert_eq!(ldollar(1, 0), ATOMS_PER_AXC); // 1 L$ = 1 AXC
        assert_eq!(to_ldollar(ATOMS_PER_AXC, 0), 1.0);
    }

    #[test]
    fn test_ldollar_dv3() {
        // dv=3: 1 AXC = 1,000 L$
        assert_eq!(to_ldollar(ATOMS_PER_AXC, 3), 1000.0);
        assert_eq!(ldollar(1000, 3), ATOMS_PER_AXC); // 1000 L$ = 1 AXC
    }

    #[test]
    fn test_format_axc() {
        assert_eq!(format_axc(10_000_000_000), "1 AXC");
        assert_eq!(format_axc(15_000_000_000), "1.5 AXC");
        assert_eq!(format_axc(500_000), "0.00005 AXC");
        assert_eq!(format_axc(0), "0 AXC");
    }

    #[test]
    fn test_format_ldollar() {
        assert_eq!(format_ldollar(ATOMS_PER_AXC, 0), "1 L$");
        assert_eq!(format_ldollar(ATOMS_PER_AXC, 3), "1,000 L$");
        assert_eq!(format_ldollar(15_000_000_000, 3), "1,500 L$");
    }

    #[test]
    fn test_format_ldollar_2dp() {
        // dv=2 → 1 AXC = 100 L$, 10^8 atoms per L$.
        assert_eq!(format_ldollar_2dp(ATOMS_PER_AXC, 2), "100 L$"); // whole → no decimals
        assert_eq!(format_ldollar_2dp(100_000_000, 2), "1 L$");     // exactly 1 L$
        assert_eq!(format_ldollar_2dp(105_000_000, 2), "1.05 L$");  // two-decimal cents
        assert_eq!(format_ldollar_2dp(100_000_000_000, 2), "1,000 L$"); // commas
        // Truncated, never rounded up (don't overstate value).
        assert_eq!(format_ldollar_2dp(109_999_999, 2), "1.09 L$");
        // Sub-0.01 L$ amounts now show the REAL figure at up to 4dp
        // (truncated), not a vague "< 0.01". 10^8 atoms per L$ at dv=2.
        assert_eq!(format_ldollar_2dp(340_000, 2), "0.0034 L$"); // 0.0034 L$
        assert_eq!(format_ldollar_2dp(300_000, 2), "0.003 L$");  // trailing zero trimmed
        assert_eq!(format_ldollar_2dp(10_000, 2), "0.0001 L$");  // smallest 4dp value
        // Below 4dp granularity → precise floor marker, never "0.00".
        assert_eq!(format_ldollar_2dp(1, 2), "< 0.0001 L$");
        // dv≥10 → 1 atom per L$, no fractional part.
        assert_eq!(format_ldollar_2dp(5, 10), "5 L$");
    }

    #[test]
    fn test_parse_axc() {
        assert_eq!(parse_axc("1").unwrap(), 10_000_000_000);
        assert_eq!(parse_axc("1.5").unwrap(), 15_000_000_000);
        assert_eq!(parse_axc("0.00005").unwrap(), 500_000);
        assert_eq!(parse_axc("1.5 AXC").unwrap(), 15_000_000_000);
    }

    #[test]
    fn test_parse_ldollar() {
        assert_eq!(parse_ldollar("1,000", 3).unwrap(), ATOMS_PER_AXC);
        assert_eq!(parse_ldollar("1,500 L$", 3).unwrap(), 15_000_000_000);
    }

    #[test]
    fn test_small_amount_conversion() {
        // A sub-cent sample amount (this is the dust-floor magnitude, but the
        // dust floor itself is Core's policy, not a denomination constant).
        let atoms = 500_000u64;
        assert_eq!(to_axc(atoms), 0.00005);
        assert_eq!(format_axc(atoms), "0.00005 AXC");
    }

    #[test]
    fn test_format_with_commas() {
        assert_eq!(format_with_commas(1000), "1,000");
        assert_eq!(format_with_commas(1000000), "1,000,000");
        assert_eq!(format_with_commas(100), "100");
        assert_eq!(format_with_commas(0), "0");
    }
}
