#[macro_use]
extern crate lazy_static;

include!(concat!(env!("OUT_DIR"), "/adjacency_data.rs"));
include!(concat!(env!("OUT_DIR"), "/frequency_data.rs"));

pub mod matching;
pub mod result;
pub mod scoring;
