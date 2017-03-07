use std::env;
use std::cmp::Ordering;
use std::iter::Iterator;

include!(concat!(env!("OUT_DIR"), "/adjacency_data.rs"));
include!(concat!(env!("OUT_DIR"), "/frequency_data.rs"));

#[derive(Eq, Clone, Debug)]
pub struct BaseMatch {
    pattern: String,
    start: usize,
    end: usize,
    token: String,
}

impl Ord for BaseMatch {
    fn cmp(&self, other: &BaseMatch) -> Ordering {
        let t1 = (self.start, self.end);
        let t2 = (other.start, other.end);
        t1.cmp(&t2)
    }
}

impl PartialOrd for BaseMatch {
    fn partial_cmp(&self, other: &BaseMatch) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for BaseMatch {
    fn eq(&self, other: &BaseMatch) -> bool {
        (self.start, self.end) == (other.start, other.end)
    }
}



/// Matches the password against every matcher returning the matches
pub fn omnimatch(password: &str) -> Vec<BaseMatch> {
    master_dictionary_match(password)
}


fn master_dictionary_match(password: &str) -> Vec<BaseMatch> {

    let default_dicts = vec![FEMALE_NAMES,
                             MALE_NAMES,
                             SURNAMES,
                             PASSWORDS,
                             ENGLISH_WIKIPEDIA,
                             US_TV_AND_FILM];

    default_dicts.iter()
                 .map(|x| dictionary_match(password, x))
                 .flat_map(|x| x.into_iter())
                 .collect::<Vec<BaseMatch>>()
}

fn dictionary_match(password: &str, 
                        dictionary: &[&'static str]) -> Vec<BaseMatch> {

    let mut matches: Vec<BaseMatch> = Vec::new();
    let lower = password.to_lowercase();
    for i in 0..password.len() {
        for j in i..password.len() + 1 {
            let slice = &lower[i..j];
            if let Some(pass) = dictionary.iter().position(|&x| x == slice) {
                matches.push(BaseMatch {
                    pattern: String::from("Dictionary"),
                    start: i,
                    end: j,
                    token: slice.to_string(),
                });
                return matches;
            }
        }
    }
    matches.sort();
    matches
}
