//! Security vulnerability tests.
//!
//! Verifies that the parser is safe against known XML attack vectors:
//! entity expansion (billion-laughs / XML bomb) and external entity
//! injection (XXE). Each test must complete quickly and must not read
//! filesystem paths from entity references.

pub mod vulnerabilities;
