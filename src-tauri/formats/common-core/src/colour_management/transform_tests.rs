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

//! Unit tests for transform.rs.

use super::ColourContext;
use crate::colour_management::colour::Colour;
use crate::colour_management::profile::IccProfileStore;
use crate::colour_management::space::{
    BuiltInProfile, ColourSpace, DocumentColourSettings, IccProfileRef,
};

fn srgb_ctx() -> ColourContext {
    let settings = DocumentColourSettings::default();
    let mut store = IccProfileStore::new();
    ColourContext::new_for_display(&settings, &mut store).unwrap()
}

#[test]
fn construct_srgb_context_succeeds() {
    let _ = srgb_ctx();
}

#[test]
fn srgb_red_is_identity() {
    let mut ctx = srgb_ctx();
    let result = ctx.convert(&Colour::Rgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    });
    assert!((result[0] - 1.0).abs() < 1e-5);
    assert!(result[1].abs() < 1e-5);
    assert!(result[2].abs() < 1e-5);
    assert!((result[3] - 1.0).abs() < 1e-5);
}

#[test]
fn srgb_grey_is_identity() {
    let mut ctx = srgb_ctx();
    let result = ctx.convert(&Colour::Rgb {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        a: 1.0,
    });
    for (i, &ch) in result.iter().enumerate().take(3) {
        assert!((ch - 0.5).abs() < 1e-5, "channel {} mismatch: {}", i, ch);
    }
}

#[test]
fn construct_cmyk_iso_coated_context_succeeds() {
    let settings = DocumentColourSettings {
        working_space: ColourSpace::Cmyk {
            profile: IccProfileRef::BuiltIn(BuiltInProfile::IsoCoatedV2),
        },
        ..Default::default()
    };
    let mut store = IccProfileStore::new();
    assert!(ColourContext::new_for_display(&settings, &mut store).is_ok());
}

#[test]
fn cmyk_paper_white_converts_near_white() {
    let settings = DocumentColourSettings {
        working_space: ColourSpace::Cmyk {
            profile: IccProfileRef::BuiltIn(BuiltInProfile::IsoCoatedV2),
        },
        ..Default::default()
    };
    let mut store = IccProfileStore::new();
    let mut ctx = ColourContext::new_for_display(&settings, &mut store).unwrap();
    let result = ctx.convert(&Colour::Cmyk {
        c: 0.0,
        m: 0.0,
        y: 0.0,
        k: 0.0,
        alpha: 1.0,
    });
    for (i, &ch) in result.iter().enumerate().take(3) {
        assert!(
            (ch - 1.0).abs() < 0.05,
            "channel {} = {} (expected ~1.0)",
            i,
            ch
        );
    }
    assert!((result[3] - 1.0).abs() < 1e-5);
}

#[test]
fn cmyk_rich_black_converts_near_black() {
    let settings = DocumentColourSettings {
        working_space: ColourSpace::Cmyk {
            profile: IccProfileRef::BuiltIn(BuiltInProfile::IsoCoatedV2),
        },
        ..Default::default()
    };
    let mut store = IccProfileStore::new();
    let mut ctx = ColourContext::new_for_display(&settings, &mut store).unwrap();
    let result = ctx.convert(&Colour::Cmyk {
        c: 0.0,
        m: 0.0,
        y: 0.0,
        k: 1.0,
        alpha: 1.0,
    });
    for (i, &ch) in result.iter().enumerate().take(3) {
        assert!(ch < 0.05, "channel {} = {} (expected ~0.0)", i, ch);
    }
}

#[test]
fn cache_size_increases_on_first_call() {
    let colour = Colour::Cmyk {
        c: 0.5,
        m: 0.3,
        y: 0.2,
        k: 0.1,
        alpha: 1.0,
    };
    let settings = DocumentColourSettings {
        working_space: ColourSpace::Cmyk {
            profile: IccProfileRef::BuiltIn(BuiltInProfile::IsoCoatedV2),
        },
        ..Default::default()
    };
    let mut store = IccProfileStore::new();
    let mut ctx = ColourContext::new_for_display(&settings, &mut store).unwrap();
    assert_eq!(ctx.cache_size(), 0);
    let r1 = ctx.convert(&colour);
    assert_eq!(ctx.cache_size(), 1);
    let r2 = ctx.convert(&colour);
    assert_eq!(ctx.cache_size(), 1);
    assert_eq!(r1, r2);
}

#[test]
fn convert_batch_matches_individual() {
    let mut ctx = srgb_ctx();
    let colours = vec![
        Colour::Rgb {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
        Colour::Rgb {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        },
    ];
    let batch = ctx.convert_batch(&colours);
    let mut ctx2 = srgb_ctx();
    for (i, c) in colours.iter().enumerate() {
        let single = ctx2.convert(c);
        assert_eq!(batch[i], single, "mismatch at index {}", i);
    }
}
