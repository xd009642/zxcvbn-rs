#[macro_use]
extern crate lazy_static;
extern crate regex;

include!(concat!(env!("OUT_DIR"), "/adjacency_data.rs"));
include!(concat!(env!("OUT_DIR"), "/frequency_data.rs"));

pub mod matching;
pub mod result;
pub mod scoring;
