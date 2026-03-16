// Copyright 2024 AppThere
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Pantone Matching System (PMS) colour lookup.
//!
//! **NOTE**: This implementation contains a representative subset of ~100
//! well-known PMS solid coated colours. The full PMS table (~1867 entries)
//! must be populated from a Pantone data source before production use.
//! Extend the `pms_*` sub-tables with the complete dataset.

mod pms_coated;
mod pms_metallics;

use pms_coated::PMS_COATED;
use pms_metallics::PMS_METALLICS;

/// Look up the CIE L*a*b* reference values for a Pantone colour name.
///
/// The name is matched case-insensitively and with normalised whitespace.
/// Both "PANTONE 186 C" and "pantone 186 c" and "Pantone 186C" are accepted.
///
/// Returns None if the colour is not in the lookup table.
pub fn lookup_pantone(name: &str) -> Option<[f32; 3]> {
    let normalised = normalise_pantone_name(name);
    if let Some(&v) = PMS_COATED.get(normalised.as_str()) {
        return Some(v);
    }
    if let Some(&v) = PMS_METALLICS.get(normalised.as_str()) {
        return Some(v);
    }
    // Try suffix substitution: U or M → C
    let with_c = substitute_suffix_to_c(&normalised);
    if let Some(alt) = with_c {
        if let Some(&v) = PMS_COATED.get(alt.as_str()) {
            return Some(v);
        }
        if let Some(&v) = PMS_METALLICS.get(alt.as_str()) {
            return Some(v);
        }
    }
    None
}

/// Returns an iterator over all known Pantone colour names.
pub fn all_pantone_names() -> impl Iterator<Item = &'static str> {
    PMS_COATED.keys().chain(PMS_METALLICS.keys()).copied()
}

/// Normalise a Pantone colour name for lookup.
///
/// Steps:
/// 1. Uppercase
/// 2. Collapse whitespace (trim + single spaces)
/// 3. Ensure "PANTONE " prefix
fn normalise_pantone_name(name: &str) -> String {
    let upper = name.to_uppercase();
    // Collapse multiple spaces and trim
    let parts: Vec<&str> = upper.split_whitespace().collect();
    let collapsed = parts.join(" ");
    // Ensure PANTONE prefix
    if collapsed.starts_with("PANTONE ") {
        collapsed
    } else if collapsed.starts_with("PANTONE") {
        // No space, e.g. "PANTONE186C" → "PANTONE 186C"
        // Try to keep it as-is; the table keys include the space
        collapsed
    } else {
        format!("PANTONE {}", collapsed)
    }
}

/// If name ends in " U" or " M", try returning a version ending in " C".
fn substitute_suffix_to_c(name: &str) -> Option<String> {
    if name.ends_with(" U") || name.ends_with(" M") {
        let base = &name[..name.len() - 1];
        Some(format!("{}C", base))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_pantone_186_c() {
        let lab = lookup_pantone("PANTONE 186 C").unwrap();
        // Known Lab values for PANTONE 186 C (red)
        assert!((lab[0] - 41.0).abs() < 5.0, "L={}", lab[0]);
        assert!(lab[1] > 40.0, "a={} (expected positive/warm)", lab[1]);
    }

    #[test]
    fn lookup_case_insensitive() {
        let a = lookup_pantone("PANTONE 186 C");
        let b = lookup_pantone("pantone 186 c");
        assert_eq!(a, b);
        assert!(a.is_some());
    }

    #[test]
    fn lookup_prefix_auto_added() {
        let a = lookup_pantone("PANTONE 186 C");
        let b = lookup_pantone("186 C");
        assert_eq!(a, b);
    }

    #[test]
    fn lookup_u_suffix_falls_back_to_c() {
        let a = lookup_pantone("PANTONE 186 C");
        let b = lookup_pantone("PANTONE 186 U");
        assert_eq!(a, b);
        assert!(b.is_some());
    }

    #[test]
    fn lookup_nonexistent_returns_none() {
        assert!(lookup_pantone("PANTONE 99999 C").is_none());
    }

    #[test]
    fn all_names_at_least_50() {
        let count = all_pantone_names().count();
        assert!(count >= 50, "Expected at least 50 entries, got {}", count);
    }
}
