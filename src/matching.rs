use std::collections::HashMap;
use std::cmp::Ordering;
use std::cmp;
use std::iter::Iterator;
use fancy_regex::Regex as FancyRegex;
use regex::Regex;
use chrono::{NaiveDate, Datelike, Local};
use scoring;
use keygraph_rs::*;

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
pub struct L33tData {
    /// Hashmap containing a key of l33t characters and a string of the characters
    /// they replace
    pub l33t_subs: HashMap<char, String>,

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
        l33t: Option<L33tData>,
    },
    Spatial {
        graph: String,
        turns: usize,
        shifted_count: usize,
    },
    Repeat {
        base_token: String,
        base_guesses: u64,
        repeat_count: usize,
    },
    Sequence {
        name: String,
        space: u32,
        ascending: bool,
    },
    Regex {
        name: String,
    },
    Date {
        separator: char,
        date: NaiveDate, 
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

pub fn matches_from_all_dicts(password: &str, 
                              matcher: &Fn(&str, &str, &[&str])->Vec<BaseMatch>) -> Vec<BaseMatch> {
    
    let dicts:HashMap<&str, &[&str]> = {
        let mut m = HashMap::new();
        m.insert("Female names", FEMALE_NAMES);
        m.insert("Male names", MALE_NAMES);
        m.insert("Surnames", SURNAMES);
        m.insert("Passwords", PASSWORDS);
        m.insert("Wikipedia", ENGLISH_WIKIPEDIA);
        m.insert("TV and Film", US_TV_AND_FILM);
        m
    };

    dicts.iter()
         .map(|(&k, &v)| matcher(password, k, v))
         .flat_map(|x| x.into_iter())
         .collect::<Vec<BaseMatch>>()
}

/// Matches the password against every matcher returning the matches
pub fn omnimatch(password: &str) -> Vec<BaseMatch> {
    
    let default_regex:HashMap<String, Regex> = {
        let mut m = HashMap::new();
        m.insert(String::from("recent year"), 
                 Regex::new(r"19\d\d|200\d|201\d").unwrap());
        m
    };

    let mut result:Vec<BaseMatch> = Vec::new();

    result.append(&mut matches_from_all_dicts(password, &dictionary_match));
    result.append(&mut matches_from_all_dicts(password, &reverse_dictionary_match));
    result.append(&mut matches_from_all_dicts(password, &l33t_match));
    result.append(&mut sequence_match(password));
    result.append(&mut regex_match(password, default_regex));
    result.append(&mut date_match(password));
    result.append(&mut repeat_match(password));
    result.append(&mut spatial_match(password));
    
    result.sort();
    result
}


fn dictionary_match(password: &str, 
                    dictionary_name: &str, 
                    dictionary: &[&str]) -> Vec<BaseMatch> {

    let mut matches: Vec<BaseMatch> = Vec::new();
    let lower = password.to_lowercase();
    for i in 0..password.len() {
        for j in i..password.len() {
            let slice = &lower[i..j + 1];
            if let Some(pass) = dictionary.iter().position(|&x| x == slice) {
                let dict = MatchData::Dictionary {
                    matched_word: slice.to_string(),
                    rank: pass + 1,
                    dictionary_name: dictionary_name.to_string(),
                    reversed: false,
                    l33t: None,
                };
                matches.push(BaseMatch {
                    pattern: String::from("Dictionary"),
                    start: i,
                    end: j,
                    token: password[i..j+1].to_string(),
                    data: dict,
                });
            }
        }
    }
    matches.sort();
    matches
}

#[test]
fn dictionary_test() {
    let m = dictionary_match("password", "test", &["pass", "password", "dave"]);
    assert_eq!(m.len(), 2);
    for temp in m.iter() {
        match temp.data {
            // Simple test
            MatchData::Dictionary{ref matched_word, ..} => assert!(matched_word != "dave"),
            _ => assert!(false),
        }
    }
}

pub fn reverse_dictionary_match(password: &str,
                                dictionary_name: &str,
                                dictionary: &[&str]) -> Vec<BaseMatch> {
    let length = password.chars().count();
    let reversed = password.chars().rev().collect::<String>();

    let mut matches = dictionary_match(reversed.as_ref(), 
                                       dictionary_name,
                                       dictionary);
    for m in matches.iter_mut() {

        m.token = m.token.chars().rev().collect::<String>();
        let (start, end) = (length - 1 - m.end, length - 1 - m.start);
        m.start = start;
        m.end = end;

        match m.data {
            MatchData::Dictionary{ref mut reversed, ..} => *reversed = true,
            _ => {}
        }
    }
    matches.sort();
    matches
}

#[test]
fn reverse_test() {
    let m = reverse_dictionary_match("drowssap", "test", &["password"]);
    assert_eq!(m.len(), 1);

    let ref temp = m[0];
    assert_eq!("drowssap", temp.token);
    match temp.data {
        MatchData::Dictionary{ref matched_word, ref rank, 
            ref reversed, ref l33t, ..} => {
            assert_eq!(*reversed, true);
            assert_eq!(*matched_word, "password");
            assert_eq!(*rank, 1);
            assert_eq!(*l33t, None);
        }
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
        }
        None => *c,
    }
}


fn check_l33t_sub(password: &str, 
                  sub: &str, 
                  dictionary_name: &str,
                  dictionary: &[&str]) -> Vec<BaseMatch> {
    let mut tm = dictionary_match(sub, dictionary_name, dictionary);
    for m in tm.iter_mut() {
        m.token = password[m.start..(m.end + 1)].to_string();
        match m.data {
            MatchData::Dictionary{ref mut l33t, ref matched_word, ..} => {
                let mut tmap:HashMap<char, String> = HashMap::new();
                for (k, v) in m.token.chars().zip(matched_word.chars()) {
                    if k == v {
                        continue;
                    }
                    if tmap.contains_key(&k) {
                        let c_as_s = v.to_string();
                        let ref mut value = tmap.get_mut(&k).unwrap();
                        if false == value.contains(v) {
                            value.push_str(c_as_s.as_ref());
                        }
                    } else {
                        tmap.insert(k, v.to_string());
                    }
                }
                *l33t = Some(L33tData { l33t_subs: tmap });
            },
            _ => {},
        }
    }
    tm
}

/// l33t match assumes that a symbol which can mean multiple letters will only
/// be used for one of those letters during a match.
/// Behaviour slightly differs from dropbox on this currently
pub fn l33t_match(password: &str, 
                  dictionary_name: &str,
                  dictionary: &[&str]) -> Vec<BaseMatch> {

    let mut matches: Vec<BaseMatch> = Vec::new();

    // First we do all the simple subs. Then go through permutations
    let partial_sub: String = password.chars()
                                      .map(|c| replace_single_l33t_char(&c))
                                      .collect();

    let remaining_l33ts = partial_sub.chars()
                                     .fold(0u32, |acc, c| acc + L33T_TABLE.contains_key(&c) as u32);

    if remaining_l33ts == 0 && partial_sub != password {

        let mut tm = check_l33t_sub(password, partial_sub.as_ref(), 
                                    dictionary_name, dictionary);
        matches.append(&mut tm);

    } else if remaining_l33ts > 0 {
        let subtable = L33T_TABLE.iter()
                                 .filter(|&(k, _)| partial_sub.contains(*k))
                                 .map(|(k, v)| (*k, *v))
                                 .collect::<Vec<(char, &str)>>();

        let sizes = subtable.iter()
                            .map(|&(_, v)| (*v).chars().count())
                            .collect::<Vec<usize>>();

        let mut current = 0;
        let mut indexes: Vec<usize> = vec![0; sizes.len()];
        while current != sizes.len() {

            let sub = subtable.iter()
                              .enumerate()
                              .map(|(i, &(k, v))| (v.chars().nth(indexes[i]).unwrap(), k))
                              .collect::<HashMap<char, char>>();
            
            if sub.len() == sizes.len() {
                let sub = sub.iter()
                             .map(|(k, v)| (*v, *k))
                             .collect::<HashMap<char, char>>();

                let full_sub = partial_sub.chars()
                                          .map(|c| {
                                              match sub.get(&c) {
                                                  Some(v) => *v,
                                                  None => c,
                                              }
                                          })
                                          .collect::<String>();
                let mut tm = check_l33t_sub(password, full_sub.as_ref(), 
                                            dictionary_name, dictionary);
                matches.append(&mut tm);
            }

            indexes[current] += 1;
            if indexes[current] == sizes[current] {
                indexes[current] = 0;
                current += 1;
                if current < sizes.len() {
                    indexes[current] += 1;
                }
            }
        }
    }
    matches.sort();
    matches
}

#[test]
fn l33t_match_test() {
    let m = l33t_match("pa$$w0rd", "t3st", &["password", "pass"]);
    assert_eq!(m.len(), 2);
    
    for temp in m.iter() {
        println!("{:?}\n", temp);
        match temp.data {
            MatchData::Dictionary{ref l33t, ..} => {
                assert!(l33t.is_some());
            },
            _ => assert!(false),
        }
    }

    let m = l33t_match("!llus1on", "t3st", &["illusion"]);
    assert_eq!(m.len(), 0);
}

fn sequence_update(token:&str, 
                   i:usize, 
                   j:usize, 
                   delta:i16) -> Option<BaseMatch> {

    let mut result:Option<BaseMatch> = None;
    let max_delta = 5;
    if (j as i32 - i as i32) > 1 || delta.abs() == 1 {
        if 0 < delta.abs() && delta.abs() <= max_delta {
            let lower = Regex::new(r"^[a-z]+$").unwrap();
            let upper = Regex::new(r"^[A-Z]+$").unwrap();
            let digits = Regex::new(r"^\d+$").unwrap();
            
            let(name, space) = if lower.is_match(token) {
                ("lower".to_string(), 26u32)
            } else if upper.is_match(token) {
                ("upper".to_string(), 26u32)
            } else if digits.is_match(token) {
                ("digits".to_string(), 10u32)
            } else {
                ("unicode".to_string(), 26u32)
            };
            let data = MatchData::Sequence { 
                name:name, 
                space:space, 
                ascending: delta>0
            };
            let res = BaseMatch{ 
                pattern: String::from("Sequence"),
                start: i,
                end: j,
                token: token.to_string(),
                data: data
            };
            result = Some(res);
        }
    }
    
    result
}

pub fn sequence_match(password: &str) -> Vec<BaseMatch> {
    let mut matches:Vec<BaseMatch> = Vec::new();
    
    let mut i = 0;
    let mut last_delta:Option<i16> = None;
    let length = password.chars().count();

    for k in 1..length {
        let mut chars = password[(k-1)..(k+1)].chars();
        // Prevent overflow/underflow
        let delta = - (chars.next().unwrap() as i16) + 
            (chars.next().unwrap() as i16);
        if last_delta.is_none() {
            last_delta = Some(delta);
        }
        match last_delta {
            Some(ld) if ld == delta => continue,
            _ => {},
        }
        let j = k - 1;
        match sequence_update(&password[i..j+1], i, j, last_delta.unwrap()) {
            Some(r) => matches.push(r),
            None => {},
        }
        i = j;
        last_delta = Some(delta);
    }
    if let Some(ld) = last_delta {
        match sequence_update(&password[i..length], i, length, ld) {
            Some(r) => matches.push(r),
            None => {},
        }
    }
    
    matches
}


#[test]
fn sequence_test() {
    let pass = "123456789";
    let matches = sequence_match(pass);
    assert_eq!(1, matches.len());
    let m = matches.iter().nth(0).unwrap();
    assert_eq!(m.pattern, "Sequence");
    assert_eq!(m.start, 0);
    assert_eq!(m.end, 9);
    assert_eq!(m.token, "123456789");
    match m.data {
        MatchData::Sequence{ref name, ref space, ref ascending} => {
            assert_eq!(*name, "digits");
            assert_eq!(*space, 10);
            assert_eq!(*ascending, true);
        },
        _ => assert!(false),
    }
}


pub fn regex_match(password: &str, 
                   regexes: HashMap<String, Regex>) -> Vec<BaseMatch> {
    let mut result: Vec<BaseMatch> = Vec::new();
    
    for (name, reg) in regexes.iter() {
        if let Some(mat) = reg.find(password) {
            let metadata = MatchData::Regex{ name:name.clone() };
            let rmatch = BaseMatch {
                pattern: String::from("Regex"),
                start: mat.start(),
                end: mat.end(),
                token: password[mat.start()..mat.end()].to_string(),
                data: metadata,
            };
            result.push(rmatch);
        }
    }
    result
}

fn map_ints_to_dmy(vals: &[i32; 3]) -> Option<NaiveDate> {
    let mut result:Option<NaiveDate> = None;
    const MIN_YEAR:i32 = 1000;
    const MAX_YEAR:i32 = 2050;

    
    if vals[1] < 32 || vals[1] > 0 {
        let mut in_range = true;
        let mut over_12 = 0;
        let mut over_31 = 0;
        let mut under_1 = 0;
        for i in vals.into_iter() {
            match *i {
                // Relies on fact ints have been parsed into valid magnitudes
                99 ... MIN_YEAR | MAX_YEAR ... 9999 => {
                    in_range = false;
                },
                _ if *i > 31 => over_31 += 1,
                _ if *i > 12 => over_12 += 1,
                _ if *i < 1 => under_1 += 1,
                _ => {},
            }
        }
        if in_range || over_31 < 2 || over_12 != 3 || under_1 < 2 {
            let possible_splits = [(vals[2], (vals[0], vals[1])),
                                   (vals[0], (vals[1], vals[2]))];

            for &(year, dm) in possible_splits.into_iter() {
                if MIN_YEAR <= year && year <= MAX_YEAR {
                    if let Some(date) = map_ints_to_dm(&dm) {
                        result = date.with_year(year);
                    }
                }
            }
            if result.is_none() {
                for &(year, dm) in possible_splits.into_iter() {
                    if let Some(date) = map_ints_to_dm(&dm) {
                        result = date.with_year(two_to_four_digit_year(year));
                    }
                }
            }
        }
    }
    result
}

fn map_ints_to_dm(i:&(i32, i32)) -> Option<NaiveDate> {
    let year = Local::now().year() as i32;
    // TODO Change to (1..32).contains() etc. when stable
    if 1 <= i.0 && i.0 <= 31 && 1 <= i.1 && i.1 <= 12 {
        NaiveDate::from_ymd_opt(year, i.1 as u32, i.0 as u32)
    } else if 1 <= i.1 && i.1 <= 31 && 1 <= i.0 && i.0 <= 12 {
        NaiveDate::from_ymd_opt(year, i.0 as u32, i.1 as u32)
    } else {
        None
    }
}

fn two_to_four_digit_year(year: i32) -> i32 {
    if year > 99 {
        year
    } else if year > 50 {
        year + 1900
    } else {
        year + 2000 
    }
}

pub fn date_match(password: &str) -> Vec<BaseMatch> {
    let mut result: Vec<BaseMatch> = Vec::new(); 
    let password_len = password.chars().count();

    let date_splits:HashMap<usize, Vec<(usize, usize)>> = {
        let mut m = HashMap::new();
        m.insert(4, vec![(1, 2), (2, 3)]);
        m.insert(5, vec![(1, 3), (2, 3)]);
        m.insert(6, vec![(1, 2), (2, 4), (4, 5)]);
        m.insert(7, vec![(1, 3), (2, 3), (4, 5), (4, 6)]);
        m.insert(8, vec![(2, 4), (4, 6)]);
        m
    };

    let maybe_date_no_sep = Regex::new(r"^\d{4,8}$").unwrap();
    // Can't access previous captures in matches
    let maybe_date_with_sep = Regex::new(r"^(\d{1,4})([\s/\\_.-])(\d{1,2})([\s/\\_.-])(\d{1,4})$")
        .unwrap(); 
    let ref_year = Local::now().year() as i32;
    for i in 0..(cmp::max(password_len, 3)-3) {
        for j in (i+3)..(i+8) {
            if j >= password_len {
                break;
            }
            let token = &password[i..j+1];

            if !maybe_date_no_sep.is_match(&token) {
                continue;
            }
            let mut candidates:Vec<NaiveDate> = Vec::new();
            for &(k, l) in date_splits.get(&token.chars().count()).unwrap().iter() {
                let a = token[0..k].parse();
                let b = token[k..l].parse();
                let c = token[l..].parse();
                if a.is_err() || b.is_err() || c.is_err() {
                    break;
                }
                if let Some(d) = map_ints_to_dmy(&[a.unwrap(), b.unwrap(),c.unwrap()]) {
                    candidates.push(d);
                }
            }
            if candidates.is_empty() {
                continue;
            }
            let mut best:usize = 0;
            let mut min_distance = i32::max_value();
            for (index, cand) in candidates.iter().enumerate() {
                let distance = (cand.year() - ref_year).abs();
                if distance < min_distance {
                    best = index;
                    min_distance = distance;
                }
            }
            let metadata = MatchData::Date { 
                separator:'\0', 
                date:*candidates.iter().nth(best).unwrap() 
            };
            let mat = BaseMatch { 
                pattern: String::from("Date"),
                token: token.to_string(),
                start: i,
                end: j,
                data: metadata,
            };
            result.push(mat);
        }
    }
    for i in 0..cmp::max(password_len, 6)-6 {
        for j in (i+5)..(i+10) {
            if j >= password_len {
                break;
            }
            let token = &password[i..j+1];
            if let Some(cap) = maybe_date_with_sep.captures(token) 
            {
                // Thanks to regex we know these are strings hence the lack of checks
                let dmy = &[
                    cap.get(1).unwrap().as_str().parse().unwrap(), 
                    cap.get(3).unwrap().as_str().parse().unwrap(), 
                    cap.get(5).unwrap().as_str().parse().unwrap()
                ];
                if let Some(d) = map_ints_to_dmy(dmy) {
                    let sep= cap.get(2)
                                .unwrap()
                                .as_str()
                                .chars()
                                .next()
                                .unwrap();

                    let metadata = MatchData::Date {
                        separator: sep,
                        date: d,
                    };
                    let mat = BaseMatch {
                        pattern: String::from("Date"),
                        token: token.to_string(),
                        start: i,
                        end: j,
                        data: metadata,
                    };
                    result.push(mat);
                }
            }
        } 
    } 
    result
}


#[test]
fn date_match_test() {

}


pub fn repeat_match(password: &str) -> Vec<BaseMatch> {
    let mut result:Vec<BaseMatch> = Vec::new();
    let count = password.chars().count();
    
    let greedy = FancyRegex::new(r"(.+)\1+").unwrap();
    let lazy = FancyRegex::new(r"(.+?)\1+").unwrap();
    let lazy_anchored = FancyRegex::new(r"^(.+?)\1+$").unwrap();

    let mut last_index = 0;
    while last_index < count {
        let gmatch = greedy.captures(&password[last_index..]).unwrap();
        if let Some(gcap) = gmatch {
            let (gstart, gend) = gcap.pos(0).unwrap();
            // if greedy matches lazy will
            let lcap = lazy.captures(&password[last_index..]).unwrap().unwrap();
            let (lstart, lend) = lcap.pos(0).unwrap();

            let base: String;
            let mut start = gstart + last_index;
            let mut end = gend + last_index;

            if gend - gstart > lend - lstart {
                let lamatch = lazy_anchored.captures(&password[gstart..gend]).unwrap().unwrap(); 
                base = lamatch.at(1).unwrap().to_string();
            } else {
                start = lstart + last_index;
                end = lend + last_index;
                base = lcap.at(1).unwrap().to_string();
            }
            
            let base_analysis = scoring::most_guessable_match_sequence(base.clone(), 
                                                                       omnimatch(base.as_ref()),
                                                                       false);
            let repeat_count = (end - start) / base.chars().count();
            let metadata = MatchData::Repeat {
                base_token: base,
                base_guesses: base_analysis.guesses,
                repeat_count: repeat_count
            };
            
            let data = BaseMatch {
                pattern: String::from("Repeat"),
                start: start,
                end: end,
                token: gcap.at(0).unwrap().to_string(),
                data: metadata
            };
            result.push(data);
            last_index += 1;     
        } else {
            break;
        }
    }
    result
}


#[test]
fn repeat_match_test() {
    let test = "aabaabaabaab";
    let result = repeat_match(test);
    assert_eq!(result.len(), 10);
    
    let first = result.iter().nth(0).unwrap();
    assert_eq!(first.pattern, "Repeat");
    assert_eq!(first.start, 0);
    assert_eq!(first.end, test.chars().count());
    assert_eq!(first.token, test);
    match first.data {
        MatchData::Repeat{ref base_token, ref repeat_count, ..} => {
            assert_eq!(*repeat_count, 4);
            assert_eq!(*base_token, "aab");
        },
        _=> assert!(false),
    };
    //let result = repeat_match("abcdefghijklmnopqrstuvwxyz");
}


pub fn spatial_match(password: &str) -> Vec<BaseMatch> {
    let mut result:Vec<BaseMatch> = Vec::new();

    let graphs = vec![
        &*QWERTY_US,
        &*DVORAK,
        &*STANDARD_NUMPAD,
        &*MAC_NUMPAD
    ];
    let names = vec![
        "qwerty",
        "dvorak",
        "Keypad",
        "Mac keypad"
    ];

    for (name, graph) in names.iter().zip(graphs.iter()) {
        result.append(&mut spatial_helper(password, name, graph));
    }

    result.sort();
    result
}


fn spatial_helper(password: &str, 
                  graph_name: &str, 
                  graph: &Keyboard) -> Vec<BaseMatch> {
    let mut result:Vec<BaseMatch> = Vec::new();
    let password_len = password.chars().count();
    let mut i = 0;
    while i < password_len {
        let mut j = i + 1;
        let mut turns = 0;
        let mut previous_direction: Option<&Edge> = None;
        let current_char = password.chars().nth(i).unwrap();
        let current_key = graph.find_key(current_char);
        if current_key.is_none() {
            i = i +1;
            continue;
        }
        let current_key = current_key.unwrap();
        let mut previous_key = current_key;
        let mut shift_count = current_key.is_shifted(current_char) as usize;
        
        loop {
            let mut found = false;
            if j < password_len {
                let current_char = password.chars().nth(j).unwrap();
                let current_key = graph.find_key(current_char);
                if current_key.is_some() {
                    let current_key = current_key.unwrap();
                    if let Some(dir) = graph.edge_weight(previous_key, current_key) {
                        found = true;    
                        shift_count += current_key.is_shifted(current_char) as usize;
                        if Some(dir) != previous_direction {
                            turns += 1;
                            previous_direction = Some(dir);
                        }
                    }
                    previous_key = current_key;
                }
            }
            if found {
                j += 1;
            } else {
                if j - i > 2 {
                    let data = MatchData::Spatial {
                        graph: graph_name.to_string(),
                        turns: turns,
                        shifted_count: shift_count,
                    };

                    let mat = BaseMatch {
                        pattern: String::from("Spatial"),
                        start: i,
                        end: j - 1,
                        token: password[i..j].to_string(),
                        data: data,
                    };
                    result.push(mat);
                }
                i = j;
                break;
            }
        }
    }
    result
}


#[test]
fn test_spatial_match() {
    let password = "mNbVcvBnM,.?";
    let matches = spatial_match(password);
    assert_eq!(matches.len(), 1);
    let mat = matches.iter().nth(0).unwrap();
    match mat.data {
        MatchData::Spatial{ref graph, ref turns, ref shifted_count} => {
            assert_eq!(*graph, "qwerty");
            assert_eq!(*turns, 2);
            assert_eq!(*shifted_count, 5);
        },
        _ => assert!(false),
    }
}
