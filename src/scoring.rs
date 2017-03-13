use result::PasswordResult;
use matching::{BaseMatch, MatchData};
use std::num;
use std::collections::HashMap;


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
            length: l
        });
     }

    fn unwind(&self, n: usize) -> Vec<BaseMatch> {
        let mut result: Vec<BaseMatch> = Vec::new();
        result.reserve(1);
        let mut k = n - 1;
        let mut l = 0usize;
        let mut g = u32::max_value();
        for score in self.scores.get(&k).unwrap().iter() {
            if score.g < g {
                g = score.g;
                l = score.length;
            }
        }
        while k >= 0 {
            if let Some(s) = self.scores.get(&k).unwrap().iter()
                .find(|x| x.length == l) {
                    let ref m = s.m;
                    k = m.start - 1;
                    result.insert(0, m.clone());
                    l -= 1;
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

fn bruteforce_match(password: String, start: usize, end: usize) -> BaseMatch {
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

    let mut optimal: OptimalMatch = { Default::default() };
    let chars = 0..password.len();

    let matches_by_end = chars.map(|x| matches.iter().filter(|y| y.end == x).collect::<Vec<_>>())
                              .collect::<Vec<_>>();


    for k in 0..password.len() {
        for m in matches_by_end[k].iter() {
            if m.start > 0 {
                // update
                
            } else {
                optimal.update(&password, m, 1);
            }
        }
        // Bruteforce update
    }
    optimal.unwind(password.len());
    // unwind optimal sequence

    // format result based on length
    let guesses = if password.len() == 0 {
        1u32
    } else {
        1u32
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
    1u32
}
