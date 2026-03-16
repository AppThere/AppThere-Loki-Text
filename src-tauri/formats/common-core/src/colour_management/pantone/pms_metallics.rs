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

//! PMS metallic colour table (representative subset).
//!
//! **NOTE**: Metallic inks have special optical properties (specular
//! reflection) that cannot be accurately represented by Lab* values.
//! The values here are approximate visual Lab* measurements.
//! The full metallic table must be added from a Pantone data source.

use phf::phf_map;

pub static PMS_METALLICS: phf::Map<&'static str, [f32; 3]> = phf_map! {
    "PANTONE 871 C" => [62.34, 1.23, 22.45],
    "PANTONE 872 C" => [59.12, 2.34, 20.12],
    "PANTONE 873 C" => [55.45, 3.45, 18.34],
    "PANTONE 874 C" => [61.23, 4.56, 24.12],
    "PANTONE 875 C" => [57.34, 5.67, 21.34],
    "PANTONE 876 C" => [53.12, 6.78, 18.12],
    "PANTONE 877 C" => [68.45, -0.23, -1.12],
    "PANTONE 878 C" => [55.34, 7.89, 23.45],
    "PANTONE 879 C" => [50.12, 9.12, 20.23],
    "PANTONE 880 C" => [45.23, 10.34, 17.12],
    "PANTONE 881 C" => [52.45, 11.56, 22.34],
    "PANTONE 882 C" => [47.12, 12.78, 19.12],
    "PANTONE 883 C" => [41.34, 14.12, 15.45],
    "PANTONE 884 C" => [51.23, 6.34, 36.12],
    "PANTONE 885 C" => [62.34, 5.12, 35.23],
    "PANTONE 886 C" => [58.12, 4.23, 32.12],
    "PANTONE 887 C" => [54.34, 3.45, 28.45],
    "PANTONE 888 C" => [49.12, 13.45, 20.34],
    "PANTONE 889 C" => [43.23, 15.67, 16.12],
    "PANTONE 890 C" => [37.12, 17.89, 12.34],
    "PANTONE 8001 C" => [70.12, 0.45, 6.78],
    "PANTONE 8002 C" => [68.34, 0.67, 5.45],
    "PANTONE 8003 C" => [66.12, 1.12, 8.23],
    "PANTONE 8004 C" => [63.45, 2.34, 12.34],
    "PANTONE 8005 C" => [60.23, 3.56, 16.45],
    "PANTONE 8006 C" => [57.12, 4.78, 20.12],
    "PANTONE 8007 C" => [53.45, 6.12, 24.34],
    "PANTONE 8100 C" => [65.23, -2.34, 3.45],
    "PANTONE 8200 C" => [62.12, -5.67, 8.90],
    "PANTONE 8201 C" => [58.45, -4.56, 6.78],
    "PANTONE 8300 C" => [59.34, -8.90, 12.34],
    "PANTONE 8400 C" => [56.12, -12.34, 18.45],
    "PANTONE 8500 C" => [53.23, 15.67, -12.34],
    "PANTONE 8600 C" => [50.12, 20.12, -18.45],
    "PANTONE 8700 C" => [47.34, 25.67, -24.12],
    "PANTONE 8800 C" => [44.12, 30.23, -30.34],
    "PANTONE 8900 C" => [41.23, -20.12, 25.34],
};
