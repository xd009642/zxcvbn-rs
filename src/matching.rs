use std::env;
use std::cmp::Ordering;
use std::iter::Iterator;

include!(concat!(env!("OUT_DIR"), "/adjacency_data.rs"));
include!(concat!(env!("OUT_DIR"), "/frequency_data.rs"));

lazy_static! {
    static ref L33T_TABLE: HashMap<char, &'static str> = {
        let mut m = HashMap::new();
        m.insert('a', "4@");
        m.insert('b', "8");
        m.insert('c', "({[<");
        m.insert('e', "3");
        m.insert('g', "69");
        m.insert('i', "1!|");
        m.insert('l', "1!7");
        m.insert('o', "0");
        m.insert('s', "$5");
        m.insert('t', "+7");
        m.insert('x', "%");
        m.insert('z', "2");
        m
    };
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Months {
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Days {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MatchData {
    /// Used for matches which don't require metadata.
    Plain,
    Dictionary {
        matched_word: String,
        rank: usize,
        dictionary_name: String,
        reversed: bool,
        l33t: bool,
    },
    Spatial {
        graph: String,
        turns: usize,
        shifted_count: usize,
    },
    Repeat {
        base_token: String,
        base_guesses: usize,
        repeat_count: usize,
    },
    L33t {
        l33t: bool,
    },
    Sequence {
        name: String,
        space: String,
        ascending: bool,
    },
    Regex {
        name: String,
        regex_match: String,
    },
    Date {
        separator: char,
        year: u8,
        month: Months,
        day: Days,
    },
}

#[derive(Clone, Debug, Eq)]
pub struct BaseMatch {
    pub pattern: String,
    pub start: usize,
    pub end: usize,
    pub token: String,
    pub data: MatchData,
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
        for j in i..password.len(){
            let slice = &lower[i..j+1];
            if let Some(pass) = dictionary.iter().position(|&x| x == slice) {
                let dict = MatchData::Dictionary{ 
                    matched_word: slice.to_string(),
                    rank: pass + 1,
                    dictionary_name: "UNKNOWN".to_string(),
                    reversed: false,
                    l33t: false,
                };
                matches.push(BaseMatch {
                    pattern: String::from("Dictionary"),
                    start: i,
                    end: j,
                    token: slice.to_string(),
                    data: dict,
                });
                return matches;
            }
        }
    }
    matches.sort();
    matches
}
