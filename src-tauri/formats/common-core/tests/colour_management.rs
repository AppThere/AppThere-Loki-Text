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

//! Integration tests for the colour_management module.

#![cfg(feature = "colour-management")]

use common_core::colour_management::{
    lookup_pantone, BuiltInProfile, Colour, ColourContext, ColourSpace, DocumentColourSettings,
    IccProfileRef, IccProfileStore, SwatchLibrary,
};

/// Round-trip: CMYK → display sRGB, all channels in [0,1], alpha preserved.
#[test]
fn cmyk_to_display_rgb_round_trip() {
    let settings = DocumentColourSettings {
        working_space: ColourSpace::Cmyk {
            profile: IccProfileRef::BuiltIn(BuiltInProfile::IsoCoatedV2),
        },
        ..Default::default()
    };
    let mut store = IccProfileStore::new();
    let mut ctx = ColourContext::new_for_display(&settings, &mut store).unwrap();
    let colour = Colour::Cmyk {
        c: 0.2,
        m: 0.4,
        y: 0.1,
        k: 0.05,
        alpha: 0.75,
    };
    let result = ctx.convert(&colour);
    for (i, &ch) in result.iter().enumerate() {
        assert!(
            (0.0..=1.0).contains(&ch),
            "channel {} out of range: {}",
            i,
            ch
        );
    }
    // Alpha must be preserved
    assert!(
        (result[3] - 0.75).abs() < 1e-5,
        "alpha mismatch: {}",
        result[3]
    );
}

/// Batch: convert_batch and individual convert return matching results.
#[test]
fn batch_matches_individual() {
    let settings = DocumentColourSettings::default();
    let mut store = IccProfileStore::new();
    let mut ctx = ColourContext::new_for_display(&settings, &mut store).unwrap();

    let colours: Vec<Colour> = (0..20)
        .map(|i| {
            if i % 2 == 0 {
                Colour::Rgb {
                    r: i as f32 / 20.0,
                    g: 0.5,
                    b: 0.3,
                    a: 1.0,
                }
            } else {
                Colour::Cmyk {
                    c: i as f32 / 40.0,
                    m: 0.2,
                    y: 0.1,
                    k: 0.05,
                    alpha: 1.0,
                }
            }
        })
        .collect();

    let batch = ctx.convert_batch(&colours);
    assert_eq!(batch.len(), 20);

    // Re-create context for individual conversion (cache is separate)
    let mut store2 = IccProfileStore::new();
    let mut ctx2 = ColourContext::new_for_display(&settings, &mut store2).unwrap();
    for (i, c) in colours.iter().enumerate() {
        let single = ctx2.convert(c);
        for ch in 0..4 {
            assert!(
                (batch[i][ch] - single[ch]).abs() < 1e-5,
                "colour {i}, channel {ch}: batch={} single={}",
                batch[i][ch],
                single[ch]
            );
        }
    }
}

/// Swatch + Pantone: look up PANTONE 186 C, build Spot colour, store in library.
#[test]
fn swatch_pantone_integration() {
    let lab = lookup_pantone("PANTONE 186 C").expect("PANTONE 186 C should be in table");
    let spot = Colour::Spot {
        name: "PANTONE 186 C".to_string(),
        tint: 1.0,
        lab_ref: lab,
        cmyk_fallback: Box::new(Colour::Cmyk {
            c: 0.0,
            m: 0.91,
            y: 0.76,
            k: 0.06,
            alpha: 1.0,
        }),
    };
    let mut lib = SwatchLibrary::new();
    let _id = lib.add_colour("PANTONE 186 C", spot);
    let found = lib
        .find_by_name("pantone 186 c")
        .expect("Should find case-insensitively");
    if let Colour::Spot { lab_ref, .. } = &found.colour {
        for i in 0..3 {
            assert!((lab_ref[i] - lab[i]).abs() < 1e-4, "Lab[{i}] mismatch");
        }
    } else {
        panic!("Expected Colour::Spot");
    }
}

/// Serialisation round-trip for all Colour variants.
#[test]
fn colour_serde_round_trip() {
    let variants = vec![
        Colour::Rgb {
            r: 0.5,
            g: 0.3,
            b: 0.8,
            a: 1.0,
        },
        Colour::Cmyk {
            c: 0.1,
            m: 0.2,
            y: 0.3,
            k: 0.4,
            alpha: 0.9,
        },
        Colour::Lab {
            l: 50.0,
            a: 25.0,
            b: -30.0,
            alpha: 1.0,
        },
        Colour::Spot {
            name: "PANTONE 286 C".to_string(),
            tint: 0.8,
            lab_ref: [28.12, 15.34, -58.45],
            cmyk_fallback: Box::new(Colour::Cmyk {
                c: 1.0,
                m: 0.74,
                y: 0.0,
                k: 0.01,
                alpha: 1.0,
            }),
        },
        Colour::Linked {
            id: "swatch-abc123".to_string(),
        },
    ];

    for (i, colour) in variants.iter().enumerate() {
        let json = serde_json::to_string(colour).expect("serialisation failed");
        let recovered: Colour = serde_json::from_str(&json)
            .unwrap_or_else(|e| panic!("deserialisation failed for variant {i}: {e}"));
        assert_eq!(colour, &recovered, "round-trip failed for variant {i}");
    }
}

/// DocumentColourSettings serialisation round-trip.
#[test]
fn document_colour_settings_serde_round_trip() {
    use common_core::colour_management::RenderingIntent;
    let settings = DocumentColourSettings {
        working_space: ColourSpace::Cmyk {
            profile: IccProfileRef::BuiltIn(BuiltInProfile::IsoCoatedV2),
        },
        rendering_intent: RenderingIntent::Perceptual,
        blackpoint_compensation: false,
    };
    let json = serde_json::to_string(&settings).expect("serialisation failed");
    let recovered: DocumentColourSettings =
        serde_json::from_str(&json).expect("deserialisation failed");
    assert_eq!(settings, recovered);
}
