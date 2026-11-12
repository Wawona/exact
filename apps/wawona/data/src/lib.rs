//! Wawona Exact chrome data source. M1 uses Contract-local machine rows;
//! M3 (#231) will answer Machines / Settings through this seam.

#![deny(missing_docs)]

use exact_plan::Value;
use exact_runner::{DataError, DataSource};

/// The app's data source.
#[derive(Default, Clone, Copy)]
pub struct Wawona;

impl DataSource for Wawona {
    fn query(&mut self, source: &str, _: &[Value]) -> Result<Value, DataError> {
        Err(DataError::UnknownSource(source.into()))
    }
}
