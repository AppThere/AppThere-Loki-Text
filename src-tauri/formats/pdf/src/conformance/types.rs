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

use serde::{Deserialize, Serialize};
use crate::error::PdfError;
use crate::export_settings::PdfXStandard;

/// A single conformance violation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConformanceViolation {
    /// Short rule identifier, e.g. "X1a/no-transparency".
    pub rule: String,
    /// Human-readable description of the violation.
    pub message: String,
    /// Whether the export pipeline will resolve this violation automatically.
    /// When true, the violation does not block export; the pipeline handles it.
    pub auto_fixable: bool,
}

impl ConformanceViolation {
    pub fn new(rule: impl Into<String>, message: impl Into<String>) -> Self {
        ConformanceViolation {
            rule: rule.into(),
            message: message.into(),
            auto_fixable: false,
        }
    }

    pub fn auto_fixable(rule: impl Into<String>, message: impl Into<String>) -> Self {
        ConformanceViolation {
            rule: rule.into(),
            message: message.into(),
            auto_fixable: true,
        }
    }
}

/// Result of a conformance check — either OK or a list of violations.
#[derive(Debug, Clone)]
pub struct ConformanceReport {
    pub standard: PdfXStandard,
    pub violations: Vec<ConformanceViolation>,
}

impl ConformanceReport {
    pub fn is_conformant(&self) -> bool {
        self.violations.is_empty()
    }

    pub fn into_result(self) -> Result<(), PdfError> {
        if self.is_conformant() {
            Ok(())
        } else {
            let msg = self
                .violations
                .iter()
                .map(|v| format!("[{}] {}", v.rule, v.message))
                .collect::<Vec<_>>()
                .join("; ");
            Err(PdfError::Conformance(msg))
        }
    }
}
