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

//! Helper for Flate compression of PDF streams.

use miniz_oxide::deflate::compress_to_vec_zlib;

/// Compress bytes using the Flate (Zlib) algorithm.
pub fn compress(data: &[u8]) -> Vec<u8> {
    compress_to_vec_zlib(data, 6)
}
