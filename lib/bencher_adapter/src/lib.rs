pub mod adapters;
pub mod error;
pub mod results;

use adapters::{
    cpp::google::AdapterCppGoogle,
    rust::{bench::AdapterRustBench, criterion::AdapterRustCriterion},
};
pub use adapters::{cpp::AdapterCpp, json::AdapterJson, magic::AdapterMagic, rust::AdapterRust};
use bencher_json::project::report::JsonAdapter;
pub use error::AdapterError;
pub use results::{adapter_results::AdapterResults, AdapterResultsArray};

pub trait Adapter {
    fn convert(&self, input: &str) -> Result<AdapterResults, AdapterError> {
        Self::parse(input)
    }

    fn parse(input: &str) -> Result<AdapterResults, AdapterError>;
}

impl Adapter for JsonAdapter {
    fn convert(&self, input: &str) -> Result<AdapterResults, AdapterError> {
        match self {
            JsonAdapter::Magic => AdapterMagic::parse(input),
            JsonAdapter::Json => AdapterJson::parse(input),
            JsonAdapter::Rust => AdapterRust::parse(input),
            JsonAdapter::RustBench => AdapterRustBench::parse(input),
            JsonAdapter::RustCriterion => AdapterRustCriterion::parse(input),
            JsonAdapter::Cpp => AdapterCpp::parse(input),
            JsonAdapter::CppGoogle => AdapterCppGoogle::parse(input),
        }
    }

    fn parse(input: &str) -> Result<AdapterResults, AdapterError> {
        AdapterMagic::parse(input)
    }
}
