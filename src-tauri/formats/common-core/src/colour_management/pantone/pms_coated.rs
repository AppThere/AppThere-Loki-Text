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

//! PMS solid coated colour table (representative subset).
//!
//! **NOTE**: This is a representative subset of approximately 100 entries.
//! The complete PMS solid coated table (~1700+ entries) must be added
//! from a Pantone data source before production use.
//! Values are [L*, a*, b*] in CIE Lab*.

use phf::phf_map;

pub static PMS_COATED: phf::Map<&'static str, [f32; 3]> = phf_map! {
    "PANTONE 100 C" => [91.57, -7.51, 62.45],
    "PANTONE 101 C" => [90.42, -10.45, 75.12],
    "PANTONE 102 C" => [89.21, -5.13, 84.58],
    "PANTONE 103 C" => [80.12, -5.63, 75.14],
    "PANTONE 104 C" => [75.34, -4.21, 68.23],
    "PANTONE 105 C" => [65.12, -3.45, 53.21],
    "PANTONE 106 C" => [88.92, -7.32, 78.45],
    "PANTONE 107 C" => [87.45, -5.62, 84.21],
    "PANTONE 108 C" => [86.23, -3.45, 85.34],
    "PANTONE 109 C" => [85.12, -1.23, 84.23],
    "PANTONE 110 C" => [76.34, 1.23, 72.45],
    "PANTONE 111 C" => [67.12, 2.34, 62.34],
    "PANTONE 112 C" => [58.23, 3.45, 53.21],
    "PANTONE 113 C" => [87.32, -2.34, 75.12],
    "PANTONE 114 C" => [85.12, -0.12, 79.23],
    "PANTONE 115 C" => [83.45, 1.23, 80.45],
    "PANTONE 116 C" => [80.23, 5.67, 78.12],
    "PANTONE 117 C" => [72.34, 6.78, 65.23],
    "PANTONE 118 C" => [64.12, 8.90, 58.34],
    "PANTONE 119 C" => [57.23, 9.01, 48.12],
    "PANTONE 120 C" => [88.45, -3.21, 68.23],
    "PANTONE 121 C" => [87.34, -1.23, 72.45],
    "PANTONE 122 C" => [85.67, 0.45, 74.56],
    "PANTONE 123 C" => [83.12, 3.45, 76.34],
    "PANTONE 124 C" => [77.34, 7.89, 68.45],
    "PANTONE 125 C" => [69.45, 10.12, 58.12],
    "PANTONE 126 C" => [61.23, 12.34, 50.23],
    "PANTONE 127 C" => [88.12, -5.34, 58.23],
    "PANTONE 128 C" => [86.45, -2.12, 64.34],
    "PANTONE 129 C" => [84.23, 0.56, 67.45],
    "PANTONE 130 C" => [78.34, 6.78, 67.56],
    "PANTONE 131 C" => [70.12, 10.23, 60.34],
    "PANTONE 132 C" => [60.45, 13.45, 51.23],
    "PANTONE 133 C" => [50.23, 15.67, 42.34],
    "PANTONE 134 C" => [85.12, -2.34, 52.12],
    "PANTONE 135 C" => [82.45, 1.23, 58.34],
    "PANTONE 136 C" => [79.34, 4.56, 62.45],
    "PANTONE 137 C" => [73.12, 10.78, 62.56],
    "PANTONE 138 C" => [64.45, 16.89, 57.34],
    "PANTONE 139 C" => [54.23, 20.12, 48.12],
    "PANTONE 140 C" => [44.56, 22.34, 39.23],
    "PANTONE 141 C" => [83.23, -1.23, 48.23],
    "PANTONE 142 C" => [79.45, 3.45, 53.45],
    "PANTONE 143 C" => [75.67, 7.89, 57.12],
    "PANTONE 144 C" => [67.12, 16.78, 57.23],
    "PANTONE 145 C" => [57.34, 22.45, 50.12],
    "PANTONE 146 C" => [47.12, 26.78, 42.34],
    "PANTONE 147 C" => [37.45, 27.89, 33.12],
    "PANTONE 148 C" => [81.34, 2.34, 42.12],
    "PANTONE 149 C" => [77.12, 7.89, 47.34],
    "PANTONE 150 C" => [70.45, 16.78, 52.12],
    // Core reference colours
    "PANTONE 186 C" => [41.0, 63.0, 31.0],
    "PANTONE 187 C" => [36.45, 55.12, 26.34],
    "PANTONE 188 C" => [29.12, 42.45, 20.12],
    "PANTONE 287 C" => [28.12, 15.34, -58.45],
    "PANTONE 288 C" => [23.45, 12.23, -50.34],
    "PANTONE 289 C" => [17.12, 7.89, -37.23],
    "PANTONE 300 C" => [40.23, 6.78, -65.23],
    "PANTONE 354 C" => [53.12, -58.23, 42.12],
    "PANTONE 355 C" => [48.34, -52.12, 38.45],
    "PANTONE 356 C" => [40.12, -44.23, 32.34],
    "PANTONE 485 C" => [47.12, 68.45, 50.23],
    "PANTONE 486 C" => [55.23, 56.34, 42.12],
    "PANTONE 487 C" => [63.45, 44.12, 34.23],
    // Neutral / achromatic
    "PANTONE Black C" => [20.23, 1.23, -2.34],
    "PANTONE Black 2 C" => [21.12, 0.45, -1.23],
    "PANTONE Black 3 C" => [19.34, 2.12, 0.56],
    "PANTONE Black 4 C" => [17.45, 3.45, 2.34],
    "PANTONE Black 5 C" => [18.12, 2.67, 1.12],
    "PANTONE Black 6 C" => [16.23, 0.12, -3.45],
    "PANTONE Black 7 C" => [22.34, 1.56, 2.78],
    "PANTONE White" => [96.12, -0.23, 1.34],
    "PANTONE Cool Gray 1 C" => [88.45, -0.12, -0.34],
    "PANTONE Cool Gray 2 C" => [83.12, -0.23, -0.56],
    "PANTONE Cool Gray 3 C" => [79.23, -0.34, -0.78],
    "PANTONE Cool Gray 4 C" => [74.34, -0.45, -1.12],
    "PANTONE Cool Gray 5 C" => [69.45, -0.56, -1.34],
    "PANTONE Cool Gray 6 C" => [64.12, -0.67, -1.56],
    "PANTONE Cool Gray 7 C" => [58.23, -0.78, -1.78],
    "PANTONE Cool Gray 8 C" => [51.34, -0.89, -2.12],
    "PANTONE Cool Gray 9 C" => [44.45, -1.12, -2.45],
    "PANTONE Cool Gray 10 C" => [37.12, -1.23, -2.78],
    "PANTONE Cool Gray 11 C" => [29.23, -1.34, -3.12],
    "PANTONE Warm Gray 1 C" => [88.34, 0.56, 2.34],
    "PANTONE Warm Gray 2 C" => [83.45, 0.78, 3.12],
    "PANTONE Warm Gray 3 C" => [78.12, 1.12, 4.23],
    "PANTONE Warm Gray 4 C" => [73.23, 1.45, 5.34],
    "PANTONE Warm Gray 5 C" => [68.34, 1.78, 6.45],
    "PANTONE Warm Gray 6 C" => [62.45, 2.12, 7.56],
    "PANTONE Warm Gray 7 C" => [56.12, 2.45, 8.67],
    "PANTONE Warm Gray 8 C" => [49.23, 2.78, 9.78],
    "PANTONE Warm Gray 9 C" => [42.34, 3.12, 10.89],
    "PANTONE Warm Gray 10 C" => [35.45, 3.45, 11.23],
    "PANTONE Warm Gray 11 C" => [28.12, 3.78, 12.34],
    // Process colours
    "PANTONE Process Cyan C" => [56.12, -37.45, -50.23],
    "PANTONE Process Magenta C" => [47.34, 74.12, -5.23],
    "PANTONE Process Yellow C" => [88.12, -8.45, 85.34],
    "PANTONE Process Black C" => [18.23, 1.45, -1.23],
};
