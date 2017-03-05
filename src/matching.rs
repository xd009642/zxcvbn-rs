use std::env;
include!(concat!(env!("OUT_DIR"), "/adjacency_data.rs"));
include!(concat!(env!("OUT_DIR"), "/frequency_data.rs"));



/// Matches the password against every matcher returning the matches
fn omnimatch(password: &str) -> Vec<&str> {
    let mut matches : Vec<&str> = Vec::new();

    matches.append(dictionary_match(password));

    matches
}


fn dictionary_match(password: &str) -> Vec<&str> {
    Vec::new()
}

fn custom_dictionary_match(password: &str, 
                           dictionary:[&'static str]) -> Vec<&str> {
    Vec::new()
}
