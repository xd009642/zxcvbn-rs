use std::env;

include!(concat!(env!("OUT_DIR"), "/adjacency_data.rs"));
include!(concat!(env!("OUT_DIR"), "/frequency_data.rs"));


/// Matches the password against every matcher returning the matches
pub fn omnimatch(password: &str) -> Vec<&str> {
    let mut matches : Vec<&str> = Vec::new();
    //let mut dict_matches = master_dictionary_match(password);
    master_dictionary_match(password, &mut matches);

    matches
}


fn master_dictionary_match(password: &str, matches: &mut Vec<&str>) {
    dictionary_match(password, matches, FEMALE_NAMES);
    dictionary_match(password, matches, MALE_NAMES);
    dictionary_match(password, matches, SURNAMES);
    dictionary_match(password, matches, PASSWORDS);
    dictionary_match(password, matches, ENGLISH_WIKIPEDIA);
    dictionary_match(password, matches, US_TV_AND_FILM);
}

fn dictionary_match<'a>(password: &str, matches: &mut Vec<&str>, 
                        dictionary: &[&'static str]) {
    println!("NOT IMPLEMENTED");
}
