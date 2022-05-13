pub extern crate ipdis_common as common;

pub mod server;

#[cfg(feature = "postgres")]
pub use ipdis_api_postgres::*;
