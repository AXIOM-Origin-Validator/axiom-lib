/// Codegen for the denomination constants from `protocol_denomination.toml`.
/// `denomination` OWNS ATOMS_PER_AXC + DUST_LIMIT and keeps their source of
/// truth in its own register (it is the lowest crate in the graph); every other
/// crate, dashboard, and tool pulls from `axiom_denomination::*`, never a literal.
use std::fs;
use std::path::Path;

fn parse_u64_literal(value: &str) -> Option<u64> {
    let cleaned: String = value
        .trim()
        .split('#').next().unwrap_or("")
        .trim()
        .chars()
        .filter(|c| *c != '_')
        .collect();
    cleaned.parse::<u64>().ok()
}

fn main() {
    let toml_path = Path::new("protocol_denomination.toml");
    println!("cargo:rerun-if-changed=protocol_denomination.toml");

    let content = fs::read_to_string(toml_path)
        .expect("Failed to read protocol_denomination.toml — the denomination unit register");

    let mut atoms_per_axc: Option<u64> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('[') {
            continue;
        }
        if let Some((key, value)) = trimmed.split_once('=') {
            if key.trim() == "atoms_per_axc" {
                atoms_per_axc = parse_u64_literal(value);
            }
        }
    }

    let atoms_per_axc = atoms_per_axc
        .expect("protocol_denomination.toml is missing atoms_per_axc — required for the denomination lib");

    // denomination owns ONLY the unit relationship (atom ↔ AXC ↔ L$). The dust
    // floor (minimum_tx_atoms) is a Core protocol POLICY, not a unit fact — it
    // lives in protocol_core.toml as MINIMUM_TX_ATOMS, read via axiom_core_logic.
    let generated = format!(
        "// AUTO-GENERATED from protocol_denomination.toml by build.rs — DO NOT EDIT\n\
         //\n\
         // axiom-denomination owns the atom↔AXC unit relationship. Every crate,\n\
         // dashboard, and tool MUST pull ATOMS_PER_AXC from `axiom_denomination::*`\n\
         // rather than redeclare a literal — that hand-mirror is the bug class this\n\
         // build.rs prevents.\n\n\
         pub const ATOMS_PER_AXC: u64 = {};\n",
        atoms_per_axc,
    );

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir).join("denomination_constants.rs");
    fs::write(&out_path, generated).expect("Failed to write denomination_constants.rs");
}
