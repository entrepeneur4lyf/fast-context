use ts_rs::TS;

// Re-export main structs for type generation
pub use crate::{RustworkxDiGraph, RustworkxGraph};

// Export all TypeScript types to a bindings directory
pub fn export_types() {
    std::fs::create_dir_all("bindings").unwrap();
    RustworkxGraph::export().unwrap();
    RustworkxDiGraph::export().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_typescript_types() {
        export_types();
    }
}
