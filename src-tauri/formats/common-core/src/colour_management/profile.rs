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

//! ICC profile loading and caching via lcms2.
//!
//! **IMPORTANT**: The bundled ICC profiles in the `icc/` directory are stubs.
//! Real ICC files must be substituted before colour-managed output is used in
//! production. Stub profiles cause a runtime warning and synthetic fallback
//! profiles are used instead.

use lcms2::Profile;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use super::space::{BuiltInProfile, IccProfileRef};

/// Raw ICC profile bytes, embedded at compile time.
/// STUB: Replace with real ICC files before production use.
/// The icc/ directory is at: src-tauri/formats/common-core/icc/
const PROFILE_SRGB: &[u8] =
    include_bytes!("../../icc/sRGB_IEC61966-2-1.icc");
const PROFILE_ISO_COATED_V2: &[u8] =
    include_bytes!("../../icc/ISO_Coated_v2.icc");
const PROFILE_SWOP_V2: &[u8] =
    include_bytes!("../../icc/SWOP_v2.icc");
const PROFILE_GRACOL_2006: &[u8] =
    include_bytes!("../../icc/GRACoL_2006.icc");

const STUB_MARKER: &[u8] = b"STUB_ICC_PROFILE";

/// Returns true if the byte slice is a stub profile (not a real ICC file).
pub(crate) fn is_stub(bytes: &[u8]) -> bool {
    bytes.starts_with(STUB_MARKER)
}

/// Stores loaded ICC profiles. Profiles are loaded on demand and cached.
pub struct IccProfileStore {
    /// Raw bytes for built-in profiles.
    built_in_bytes: HashMap<BuiltInProfile, &'static [u8]>,
    /// Loaded and parsed lcms2 Profile objects, keyed by IccProfileRef.
    loaded: HashMap<IccProfileRef, Profile>,
    /// Tracks which profiles are stubs (not real ICC data).
    stubs: HashSet<IccProfileRef>,
}

impl IccProfileStore {
    /// Create a new store pre-populated with built-in profile byte references.
    /// Profiles are not yet parsed — parsing happens lazily on first use.
    pub fn new() -> Self {
        let mut built_in_bytes = HashMap::new();
        built_in_bytes.insert(BuiltInProfile::SrgbIec61966, PROFILE_SRGB);
        built_in_bytes.insert(BuiltInProfile::IsoCoatedV2, PROFILE_ISO_COATED_V2);
        built_in_bytes.insert(BuiltInProfile::SwopV2, PROFILE_SWOP_V2);
        built_in_bytes.insert(BuiltInProfile::GraCol2006, PROFILE_GRACOL_2006);
        Self {
            built_in_bytes,
            loaded: HashMap::new(),
            stubs: HashSet::new(),
        }
    }

    /// Load a profile from a file path and cache it.
    /// Returns an error if the file cannot be read or is not a valid ICC profile.
    pub fn load_file(&mut self, path: &Path) -> Result<IccProfileRef, String> {
        let bytes = std::fs::read(path)
            .map_err(|e| format!("Failed to read ICC file {:?}: {}", path, e))?;
        let profile = Profile::new_icc(&bytes)
            .map_err(|_| format!("Invalid ICC profile: {:?}", path))?;
        let key = IccProfileRef::FilePath(path.to_string_lossy().into_owned());
        self.loaded.insert(key.clone(), profile);
        Ok(key)
    }

    /// Retrieve a loaded profile, loading it from bytes if not yet parsed.
    /// Returns None if the ref is not registered in this store.
    pub fn get_or_load(&mut self, r: &IccProfileRef) -> Option<&Profile> {
        if self.loaded.contains_key(r) {
            return self.loaded.get(r);
        }
        let profile = match r {
            IccProfileRef::BuiltIn(b) => {
                let bytes = *self.built_in_bytes.get(b)?;
                if is_stub(bytes) {
                    eprintln!(
                        "WARNING: Bundled ICC profile '{}' is a stub. \
                         Real profiles must be installed for accurate colour output.",
                        b.display_name()
                    );
                    self.stubs.insert(r.clone());
                    self.make_synthetic_profile(b)
                } else {
                    Profile::new_icc(bytes).ok()?
                }
            }
            IccProfileRef::FilePath(_) => return None,
        };
        self.loaded.insert(r.clone(), profile);
        self.loaded.get(r)
    }

    /// Returns the raw bytes for a built-in profile, if available.
    pub fn raw_bytes(&self, r: &BuiltInProfile) -> Option<&'static [u8]> {
        self.built_in_bytes.get(r).copied()
    }

    /// Returns true if the given profile ref is a stub.
    pub(crate) fn is_stub_ref(&self, r: &IccProfileRef) -> bool {
        if let IccProfileRef::BuiltIn(b) = r {
            if let Some(bytes) = self.built_in_bytes.get(b) {
                return is_stub(bytes);
            }
        }
        false
    }

    fn make_synthetic_profile(&self, b: &BuiltInProfile) -> Profile {
        match b {
            BuiltInProfile::SrgbIec61966 => Profile::new_srgb(),
            // For CMYK press profiles, use Lab as a placeholder.
            // This allows the store to return Some(_) but conversions via
            // this profile will not be colour-accurate. Use real ICC files
            // for production colour management.
            _ => {
                let d50 = lcms2::CIExyY { x: 0.3457, y: 0.3585, Y: 1.0 };
                Profile::new_lab4_context(lcms2::GlobalContext::new(), &d50)
                    .unwrap_or_else(|_| Profile::new_srgb())
            }
        }
    }
}

impl Default for IccProfileStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_constructs_without_panicking() {
        let _store = IccProfileStore::new();
    }

    #[test]
    fn get_or_load_srgb_returns_some() {
        let mut store = IccProfileStore::new();
        let r = IccProfileRef::BuiltIn(BuiltInProfile::SrgbIec61966);
        assert!(store.get_or_load(&r).is_some());
    }

    #[test]
    fn get_or_load_all_built_in_profiles_return_some() {
        let mut store = IccProfileStore::new();
        let profiles = [
            IccProfileRef::BuiltIn(BuiltInProfile::SrgbIec61966),
            IccProfileRef::BuiltIn(BuiltInProfile::IsoCoatedV2),
            IccProfileRef::BuiltIn(BuiltInProfile::SwopV2),
            IccProfileRef::BuiltIn(BuiltInProfile::GraCol2006),
        ];
        for r in &profiles {
            assert!(store.get_or_load(r).is_some(), "Expected Some for {:?}", r);
        }
    }

    #[test]
    fn load_file_nonexistent_returns_err() {
        let mut store = IccProfileStore::new();
        let result = store.load_file(Path::new("/nonexistent/path/profile.icc"));
        assert!(result.is_err());
    }
}
