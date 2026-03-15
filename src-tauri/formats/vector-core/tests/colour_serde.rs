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

//! Integration tests verifying the Rust ↔ TypeScript colour serialisation
//! contract. If these tests fail, update src/lib/vector/types.ts to match.

use common_core::colour_management::Colour;

#[test]
fn colour_serde_round_trip() {
    let colours = vec![
        Colour::black(),
        Colour::white(),
        Colour::from_u8_rgba(255, 128, 0, 200),
        Colour::Cmyk {
            c: 0.1,
            m: 0.9,
            y: 0.8,
            k: 0.05,
            alpha: 1.0,
        },
        Colour::Lab {
            l: 50.0,
            a: 25.0,
            b: -30.0,
            alpha: 0.8,
        },
    ];

    for colour in &colours {
        let json = serde_json::to_string(colour).unwrap();
        // Verify the discriminant field is named "type"
        assert!(
            json.contains("\"type\":"),
            "Colour JSON must use 'type' as discriminant field, got: {}",
            json
        );
        let round_tripped: Colour = serde_json::from_str(&json).unwrap();
        assert_eq!(
            colour, &round_tripped,
            "Colour did not survive JSON round-trip: {}",
            json
        );
    }
}

#[test]
fn colour_json_shape_matches_typescript_contract() {
    // These exact JSON shapes must match the TypeScript Colour union in
    // src/lib/vector/types.ts. If this test fails, update types.ts.
    let cases: Vec<(Colour, &str)> = vec![
        (
            Colour::Rgb {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            r#"{"type":"Rgb","r":1.0,"g":0.0,"b":0.0,"a":1.0}"#,
        ),
        (
            Colour::Cmyk {
                c: 0.0,
                m: 1.0,
                y: 1.0,
                k: 0.0,
                alpha: 1.0,
            },
            r#"{"type":"Cmyk","c":0.0,"m":1.0,"y":1.0,"k":0.0,"alpha":1.0}"#,
        ),
        (
            Colour::Linked {
                id: "swatch-001".to_string(),
            },
            r#"{"type":"Linked","id":"swatch-001"}"#,
        ),
    ];

    for (colour, expected_json) in cases {
        let actual_json = serde_json::to_string(&colour).unwrap();
        let actual: serde_json::Value = serde_json::from_str(&actual_json).unwrap();
        let expected: serde_json::Value = serde_json::from_str(expected_json).unwrap();
        assert_eq!(actual, expected, "JSON shape mismatch for {:?}", colour);
    }
}
