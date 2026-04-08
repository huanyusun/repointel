//! Browser-specific parsing hooks will wrap the shared IR contracts here.
//! Phase 1 keeps the web crate intentionally thin while the native pipeline stabilizes.

use ci_ir::RepoIr;

pub fn parse_browser_snapshot(_bytes: &[u8]) -> Option<RepoIr> {
    None
}
