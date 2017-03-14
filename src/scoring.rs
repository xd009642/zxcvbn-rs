use result::PasswordResult;
use matching::{BaseMatch, MatchData};
use std::num;
use std::collections::HashMap;
use std::cmp;
use regex::Regex;

const BRUTEFORCE_CARDINALITY: u32 = 10;
const MIN_GUESSES_BEFORE_GROWING_SEQUENCE: u32 = 10000;
const MIN_SUBMATCH_GUESSES_SINGLE_CHAR: u32 = 10;
const MIN_SUBMATCH_GUESSES_MULTI_CHAR: u32 = 50;


struct MatchScores {
    m: BaseMatch,
    pi: u32,
    g: u32,
    length: usize,
}

#[derive(Default)]
struct OptimalMatch {
    scores: HashMap<usize, Vec<MatchScores>>,
}

impl OptimalMatch {
    fn update(&mut self, pass: &str, m: &BaseMatch, l: usize) {
        let k = m.end;

        let mut pi = estimate_guesses(m, pass);
        if l > 1 {
            assert!(self.scores.contains_key(&(m.start - 1)));
            if let Some(score_list) = self.scores.get(&(m.start - 1)) {
                if let Some(s) = score_list.iter().find(|x| x.length == l - 1) {
                    pi *= s.pi;
                }
            }
        }
        let g = factorial(l as u32) * pi;

        if self.scores.contains_key(&k) {
            let scores = self.scores.get_mut(&k).unwrap();
            for scores in scores.iter() {
                if scores.length > l {
                    continue;
                } else if scores.g <= g {
                    return;
                }
            }
        } else {
            self.scores.insert(k, vec![]);
        }

        self.scores.get_mut(&k).unwrap().push(MatchScores {
            m: m.clone(),
            g: g,
            pi: pi,
            length: l,
        });
    }

    fn unwind(&self, n: usize) -> Vec<BaseMatch> {
        let mut result: Vec<BaseMatch> = Vec::new();
        result.reserve(1);
        let mut k = (n as i32) - 1i32;
        let mut l = 0usize;
        let mut g = u32::max_value();
        if let Some(scores) = self.scores.get(&(k as usize)) {
            for score in scores.iter() {
                if score.g < g {
                    g = score.g;
                    l = score.length;
                }
            }
        }
        while k >= 0 {
            if let Some(scores) = self.scores.get(&(k as usize)) {
                if let Some(s) = scores.iter().find(|x| x.length == l) {
                    let ref m = s.m;
                    k = (m.start as i32) - 1i32;
                    result.insert(0, m.clone());
                    l -= 1;
                }
            }
        }
        result
    }
}

fn factorial(n: u32) -> u32 {
    let result = if n < 2 {
        1
    } else {
        (2..(n + 1)).fold(1, |acc, x| acc * x)
    };
    result
}

#[test]
fn factorial_test() {
    assert!(factorial(0) == 1);
    assert!(factorial(1) == 1);
    assert!(factorial(2) == 2);
    assert!(factorial(3) == 6);
    assert!(factorial(10) == 3628800);
}

fn nCk(mut n: u32, k: u32) -> u32 {
    let result = if k > n {
        0
    } else if 0 == k {
        1
    } else {
        (1..k + 1).fold(1, |acc, d| {
            n -= 1;
            (acc * (n + 1)) / d
        })
    };
    result
}

#[test]
fn nCk_test() {
    assert!(nCk(2, 1) == 2);
    assert!(nCk(2, 2) == 1);
    assert!(nCk(2, 3) == 0);
    assert!(nCk(85, 5) == 32801517);
}


fn bruteforce_match(password: &String, start: usize, end: usize) -> BaseMatch {
    BaseMatch {
        pattern: String::from("Bruteforce"),
        start: start,
        end: end,
        token: password[start..end].to_string(),
        data: MatchData::Plain,
    }
}

pub fn most_guessable_match_sequence(password: String,
                                     matches: Vec<BaseMatch>,
                                     exclude_additive: bool)
                                     -> PasswordResult {

    let pref = password.as_str();
    let mut optimal: OptimalMatch = {
        Default::default()
    };
    let chars = 0..password.len();
    let matches_by_end = chars.map(|x| matches.iter().filter(|y| y.end == x).collect::<Vec<_>>())
                              .collect::<Vec<_>>();

    for k in 0..password.len() {
        for m in matches_by_end[k].iter() {
            if m.start > 0 {
                // update
                let lengths = optimal.scores
                    .get(&(m.start - 1))
                    .iter()
                    .flat_map(|x| x.into_iter())
                    .map(|x| x.length)
                    .collect::<Vec<usize>>();
                for l in lengths.iter() {
                    optimal.update(pref, m, l + 1);
                }
            } else {
                optimal.update(pref, m, 1);
            }
        }
        // Bruteforce update
        let bm = bruteforce_match(&password, 0, k);
        optimal.update(pref, &bm, 1);
        for i in 1..k {
            let bm = bruteforce_match(&password, i, k);

            let lengths = optimal.scores
                .get(&(i - 1))
                .iter()
                .flat_map(|x| x.into_iter())
                .map(|x| (x.length, x.m.pattern.clone()))
                .collect::<Vec<(usize, String)>>();

            for l in lengths.iter() {
                if l.1 == "Bruteforce".to_string() {
                    continue;
                }
                optimal.update(pref, &bm, l.0 + 1);
            }
        }
    }
    let optimal_seq = optimal.unwind(password.len());
    let optimal_length = optimal_seq.iter().count();
    // unwind optimal sequence
    
    // format result based on length
    let guesses = if password.len() == 0 {
        1u32
    } else {
        let mut gs = 1u32;
        if let Some(s) = optimal.scores.get(&(password.len() - 1)) {
            let ms = s.get(optimal_length);
            if ms.is_some() {
                gs = ms.unwrap().g;
            }
        }
        gs
    };
    let g_log10 = (guesses as f64).log10();

    PasswordResult {
        password: password.clone(),
        guesses: guesses,
        guesses_log10: g_log10,
        ..Default::default()
    }
}


fn estimate_guesses(m: &BaseMatch, password: &str) -> u32 {
    // Here in coffeescript they dynamically add more struct fields to the
    // match which exist in the result anyway. It just seems so wasteful.
    // gonna think of something better but until then this will suffice.

    let min_guesses = if m.token.len() < password.len() {
        if m.token.len() == 1 {
            MIN_SUBMATCH_GUESSES_SINGLE_CHAR
        } else {
            MIN_SUBMATCH_GUESSES_MULTI_CHAR
        }
    } else {
        1u32
    };
    let guesses = match m.pattern.as_str() {
        "Bruteforce" => bruteforce_guesses(&m),
        "Dictionary" => dictionary_guesses(&m),
        "Repeat" => repeat_guesses(&m),
        "Sequence" => sequence_guesses(&m),
        "Regex" => regex_guesses(&m),
        "Date" => date_guesses(&m),
        "Spatial" => spatial_guesses(&m),
        _ => 0u32,
    };
    cmp::max(guesses, min_guesses)
}


fn bruteforce_guesses(m: &BaseMatch) -> u32 {
    let min_guesses = if m.token.len() == 1 {
        MIN_SUBMATCH_GUESSES_SINGLE_CHAR + 1
    } else {
        MIN_SUBMATCH_GUESSES_MULTI_CHAR + 1
    };
    cmp::max(min_guesses,
             BRUTEFORCE_CARDINALITY.pow(m.token.len() as u32))
}

fn dictionary_guesses(m: &BaseMatch) -> u32 {
    match m.data {
        MatchData::Dictionary {ref matched_word, rank, ref dictionary_name, reversed, l33t } => {
            let urank = uppercase_variations(m);
            let l33t_rank = l33t_variations(m);
            (rank as u32) * urank * l33t_rank
        }
        _ => 0u32,
    }

}

fn uppercase_variations(m: &BaseMatch) -> u32 {
    let token = m.token.as_str();
    
    if token.to_lowercase() == token {
        return 1u32;
    }
    let first_upper = Regex::new(r"^[A-Z][^A=Z]+$").unwrap();
    let last_upper = Regex::new(r"^[^A-Z]+[A-Z]$").unwrap();
    if token.to_uppercase() == token || first_upper.is_match(token) ||
       last_upper.is_match(token) {
        return 2u32;
    }

    let ucount = token.chars().filter(|x| x.is_uppercase()).count() as u32;
    let lcount = token.chars().filter(|x| x.is_lowercase()).count() as u32;
    let mut variations = 0u32;

    for i in 1..(cmp::min(ucount, lcount)+1) {
        variations += nCk(ucount+lcount, i);
    }
    variations
}

fn l33t_variations(m: &BaseMatch) -> u32 {
    1u32
}


fn repeat_guesses(m: &BaseMatch) -> u32 {
    1u32
}

fn sequence_guesses(m: &BaseMatch) -> u32 {
    1u32
}

fn regex_guesses(m: &BaseMatch) -> u32 {
    1u32
}

fn date_guesses(m: &BaseMatch) -> u32 {
    1u32
}

fn spatial_guesses(m: &BaseMatch) -> u32 {
    1u32
}
