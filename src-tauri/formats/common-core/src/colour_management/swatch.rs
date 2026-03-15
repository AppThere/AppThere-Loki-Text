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

//! Named colour swatches and the document-level swatch library.

use super::colour::Colour;
use serde::{Deserialize, Serialize};

/// A unique identifier for a colour swatch within a document.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SwatchId(pub String);

impl SwatchId {
    /// Generate a new unique swatch ID using an atomic counter.
    /// Does not require the `uuid` crate.
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        Self(format!("swatch-{:016x}", n))
    }
}

impl Default for SwatchId {
    fn default() -> Self {
        Self::new()
    }
}

/// A named colour swatch in the document's palette.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColourSwatch {
    pub id: SwatchId,
    /// Display name, e.g. "Brand Blue" or "PANTONE 286 C".
    pub name: String,
    /// The colour value. May be any Colour variant.
    pub colour: Colour,
    /// Whether this swatch represents a spot (named) ink.
    /// If true, `colour` should be a `Colour::Spot` variant.
    pub is_spot: bool,
}

/// The document-level collection of named colour swatches.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SwatchLibrary {
    swatches: Vec<ColourSwatch>,
}

impl SwatchLibrary {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a swatch. Returns a clone of its ID.
    pub fn add(&mut self, swatch: ColourSwatch) -> SwatchId {
        let id = swatch.id.clone();
        self.swatches.push(swatch);
        id
    }

    /// Add a colour with a given name. Generates an ID automatically.
    pub fn add_colour(&mut self, name: impl Into<String>, colour: Colour) -> SwatchId {
        let id = SwatchId::new();
        let swatch = ColourSwatch {
            id: id.clone(),
            name: name.into(),
            colour,
            is_spot: false,
        };
        self.swatches.push(swatch);
        id
    }

    /// Look up a swatch by ID. Returns None if not found.
    pub fn get(&self, id: &SwatchId) -> Option<&ColourSwatch> {
        self.swatches.iter().find(|s| &s.id == id)
    }

    /// Look up a swatch by name (case-insensitive). Returns the first match.
    pub fn find_by_name(&self, name: &str) -> Option<&ColourSwatch> {
        let lower = name.to_lowercase();
        self.swatches
            .iter()
            .find(|s| s.name.to_lowercase() == lower)
    }

    /// Remove a swatch by ID. Returns true if it existed.
    pub fn remove(&mut self, id: &SwatchId) -> bool {
        let before = self.swatches.len();
        self.swatches.retain(|s| &s.id != id);
        self.swatches.len() < before
    }

    /// Update a swatch's colour in place. Returns true if found.
    pub fn update_colour(&mut self, id: &SwatchId, colour: Colour) -> bool {
        if let Some(s) = self.swatches.iter_mut().find(|s| &s.id == id) {
            s.colour = colour;
            true
        } else {
            false
        }
    }

    /// Returns all swatches in insertion order.
    pub fn all(&self) -> &[ColourSwatch] {
        &self.swatches
    }

    /// Returns only spot colour swatches.
    pub fn spot_colours(&self) -> impl Iterator<Item = &ColourSwatch> {
        self.swatches.iter().filter(|s| s.is_spot)
    }

    /// Returns the number of swatches.
    pub fn len(&self) -> usize {
        self.swatches.len()
    }

    pub fn is_empty(&self) -> bool {
        self.swatches.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_lib() -> SwatchLibrary {
        SwatchLibrary::new()
    }

    #[test]
    fn add_colour_and_get_round_trip() {
        let mut lib = make_lib();
        let id = lib.add_colour("Red", Colour::from_u8_rgb(255, 0, 0));
        let swatch = lib.get(&id).unwrap();
        assert_eq!(swatch.name, "Red");
        assert_eq!(swatch.colour, Colour::from_u8_rgb(255, 0, 0));
    }

    #[test]
    fn find_by_name_case_insensitive() {
        let mut lib = make_lib();
        lib.add_colour("Brand Blue", Colour::from_u8_rgb(0, 0, 200));
        assert!(lib.find_by_name("brand blue").is_some());
        assert!(lib.find_by_name("BRAND BLUE").is_some());
        assert!(lib.find_by_name("Brand Blue").is_some());
        assert!(lib.find_by_name("brand_blue").is_none());
    }

    #[test]
    fn remove_returns_true_for_existing() {
        let mut lib = make_lib();
        let id = lib.add_colour("X", Colour::black());
        assert!(lib.remove(&id));
        assert!(!lib.remove(&id));
    }

    #[test]
    fn update_colour_modifies_in_place() {
        let mut lib = make_lib();
        let id = lib.add_colour("Y", Colour::black());
        assert!(lib.update_colour(&id, Colour::white()));
        assert_eq!(lib.get(&id).unwrap().colour, Colour::white());
    }

    #[test]
    fn spot_colours_filter() {
        let mut lib = make_lib();
        lib.add_colour("Regular", Colour::black());
        let spot_id = SwatchId::new();
        lib.add(ColourSwatch {
            id: spot_id.clone(),
            name: "PANTONE 186 C".to_string(),
            colour: Colour::Spot {
                name: "PANTONE 186 C".to_string(),
                tint: 1.0,
                lab_ref: [41.0, 63.0, 31.0],
                cmyk_fallback: Box::new(Colour::black()),
            },
            is_spot: true,
        });
        let spots: Vec<_> = lib.spot_colours().collect();
        assert_eq!(spots.len(), 1);
        assert_eq!(spots[0].id, spot_id);
    }
}
