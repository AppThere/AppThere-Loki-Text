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

//! PDF resource management: ICC profiles, colour spaces, ExtGState.

use crate::error::PdfError;
use common_core::colour_management::{BuiltInProfile, IccProfileRef, IccProfileStore};
use fnv::FnvHashMap;
use pdf_writer::{Chunk, Ref};

/// Tracks ICC profile objects already written, keyed by a stable string ID.
pub struct ResourceTable {
    /// Maps profile key → (object ref, colour space ref).
    icc_profiles: FnvHashMap<String, (Ref, Ref)>,
    /// Maps opacity key → ExtGState ref.
    ext_g_states: FnvHashMap<u32, Ref>,
    next_ref: i32,
}

impl ResourceTable {
    pub fn new(start_ref: i32) -> Self {
        ResourceTable {
            icc_profiles: FnvHashMap::default(),
            ext_g_states: FnvHashMap::default(),
            next_ref: start_ref,
        }
    }

    pub fn alloc_ref(&mut self) -> Ref {
        let r = Ref::new(self.next_ref);
        self.next_ref += 1;
        r
    }

    /// Get or create an ICCBased colour space ref for the given profile.
    ///
    /// Returns `(icc_stream_ref, cs_array_ref)`.
    pub fn get_or_insert_icc(
        &mut self,
        profile_ref: &IccProfileRef,
        store: &mut IccProfileStore,
        chunk: &mut Chunk,
    ) -> Result<(Ref, Ref), PdfError> {
        let key = icc_key(profile_ref);
        if let Some(&refs) = self.icc_profiles.get(&key) {
            return Ok(refs);
        }

        let raw = match profile_ref {
            IccProfileRef::BuiltIn(bp) => store
                .raw_bytes(bp)
                .ok_or_else(|| PdfError::ColourProfile(format!("No raw bytes for {:?}", bp)))?,
            IccProfileRef::FilePath(p) => {
                return Err(PdfError::ColourProfile(format!(
                    "File-path ICC profiles not yet supported for PDF export: {}",
                    p
                )));
            }
        };

        let n_components = icc_components(profile_ref);
        let stream_ref = self.alloc_ref();
        let cs_ref = self.alloc_ref();

        // Write the ICC stream.
        let mut icc_stream = chunk.icc_profile(stream_ref, raw);
        icc_stream.n(n_components);
        drop(icc_stream);

        self.icc_profiles.insert(key, (stream_ref, cs_ref));
        Ok((stream_ref, cs_ref))
    }

    /// Get or create an ExtGState ref for the given opacity (0–255 quantised).
    pub fn get_or_insert_opacity(&mut self, opacity: f64, chunk: &mut Chunk) -> Ref {
        // Quantise to avoid float key issues.
        let key = (opacity.clamp(0.0, 1.0) * 255.0).round() as u32;
        if let Some(&r) = self.ext_g_states.get(&key) {
            return r;
        }
        let r = self.alloc_ref();
        let alpha = key as f32 / 255.0;
        let mut gs = chunk.ext_graphics(r);
        gs.non_stroking_alpha(alpha);
        gs.stroking_alpha(alpha);
        drop(gs);
        self.ext_g_states.insert(key, r);
        r
    }

    pub fn icc_profiles(&self) -> &FnvHashMap<String, (Ref, Ref)> {
        &self.icc_profiles
    }

    pub fn ext_g_states(&self) -> &FnvHashMap<u32, Ref> {
        &self.ext_g_states
    }
}

fn icc_key(r: &IccProfileRef) -> String {
    match r {
        IccProfileRef::BuiltIn(bp) => format!("builtin:{:?}", bp),
        IccProfileRef::FilePath(p) => format!("file:{}", p),
    }
}

fn icc_components(r: &IccProfileRef) -> i32 {
    match r {
        IccProfileRef::BuiltIn(BuiltInProfile::SrgbIec61966) => 3,
        IccProfileRef::BuiltIn(BuiltInProfile::IsoCoatedV2) => 4,
        IccProfileRef::BuiltIn(BuiltInProfile::SwopV2) => 4,
        IccProfileRef::BuiltIn(BuiltInProfile::GraCol2006) => 4,
        IccProfileRef::FilePath(_) => 3, // assume RGB for file paths
    }
}
