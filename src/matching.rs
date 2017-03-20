use std::cmp;
use std::cmp::Ordering;
use std::iter::Iterator;

include!(concat!(env!("OUT_DIR"), "/adjacency_data.rs"));
include!(concat!(env!("OUT_DIR"), "/frequency_data.rs"));

lazy_static! {
    /// This map goes the other way in the original implementation.
    /// However, this complicates the logic and requires another map to be made
    /// inside the l33t_dictionary_match. This was deemed a cleaner and simpler
    /// implementation.
    static ref L33T_TABLE: HashMap<char, &'static str> = {
        let mut m = HashMap::new();
        m.insert('4', "a");
        m.insert('@', "a");
        m.insert('8', "b");
        m.insert('(', "c");
        m.insert('{', "c");
        m.insert('[', "c");
        m.insert('<', "c");
        m.insert('3', "e");
        m.insert('6', "g");
        m.insert('9', "g");
        m.insert('1', "il");
        m.insert('!', "il");
        m.insert('|', "i");
        m.insert('7', "lt");
        m.insert('0', "o");
        m.insert('$', "s");
        m.insert('5', "s");
        m.insert('+', "t");
        m.insert('%', "x");
        m.insert('2', "z");
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
            }
        }
    }
    matches.sort();
    matches
}


pub fn reverse_dictionary_match(password: &str,
                                dictionary: &[&'static str]) -> Vec<BaseMatch> {
    let length = password.chars().count();
    let reversed = password.chars().rev().collect::<String>();

    let mut matches = dictionary_match(reversed.as_ref(), dictionary);
    for m in matches.iter_mut() {

        m.token = m.token.chars().rev().collect::<String>();
        let (start, end) = (length - 1 - m.end, length - 1 - m.start);
        m.start = start;
        m.end = end;

        match m.data {
            MatchData::Dictionary{ref mut reversed, ..} => {
                *reversed = true        
            },
            _ => {},
        }
    }
    matches.sort();
    matches
}

#[test]
fn reverse_test() {
    let m = reverse_dictionary_match("drowssap", &["password"]);
    assert_eq!(m.len(), 1);

    let ref temp = m[0];
    assert_eq!("drowssap", temp.token);
    match temp.data {
        MatchData::Dictionary{ref matched_word, ref rank, ref dictionary_name, 
            ref reversed, ref l33t} => {
            assert_eq!(*reversed, true);
            assert_eq!(*matched_word, "password");
            assert_eq!(*rank, 1);
            assert_eq!(*l33t, false);
        },
        _ => assert!(false),
    }
}


fn replace_single_l33t_char(c: &char) -> char {
    let res = L33T_TABLE.get(c);
    match res {
        Some(s) => { 
            if s.chars().count() == 1 {
                s.chars().nth(0).unwrap_or(*c)
            } else {
                *c
            }
        },
        None => *c
    }
}

/// l33t match assumes that a symbol which can mean multiple letters will only
/// be used for one of those letters during a match.
pub fn l33t_match(password: &str,
                  dictionary: &[&'static str]) -> Vec<BaseMatch> {
    
    let mut matches: Vec<BaseMatch> =Vec::new();

    // First we do all the simple subs. Then go through permutations
    let partial_sub:String = password.chars()
                                     .map(|c| replace_single_l33t_char(&c))
                                     .collect();

    let remaining_l33ts = partial_sub.chars()
                                     .fold(0u32, |acc, c| acc + L33T_TABLE.contains_key(&c) as u32);
    
    if remaining_l33ts == 0 && partial_sub != password {
        let mut tm = dictionary_match(partial_sub.as_ref(), dictionary);
        for m in tm.iter_mut() {
            // Fix the metadata.
            m.token = password[m.start..(m.end+1)].to_string();
            match m.data {
                MatchData::Dictionary{ref mut l33t, ..} => {
                    *l33t = true;
                },
                _ => {},
            }
        }
        matches.append(&mut tm);
    } else if remaining_l33ts > 0 {
        // Do the permutations!
        let sub_table = L33T_TABLE.iter()
                                  .filter(|&(k, v)| partial_sub.contains(*k))
                                  .collect::<Vec<(&char, &&str)>>();
    }
    matches.sort();
    matches
}

#[test]
fn l33t_match_test() {
    let m = l33t_match("pa$$w0rd", &["password", "pass"]);
    assert_eq!(m.len(), 2);

    for temp in m.iter() {

        match temp.data {
            MatchData::Dictionary{ref l33t, ..} => assert_eq!(*l33t, true),
            _ => assert!(false),
        }
    }
}
