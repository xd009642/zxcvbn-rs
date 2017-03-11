use result::PasswordResult;
use matching::BaseMatch;


struct OptimalMatch {
    m: Vec<Option<BaseMatch>>,
    pi: Vec<u32>,
    g: Vec<u32>,
}

impl OptimalMatch {
    fn new(length: usize) -> OptimalMatch {
        OptimalMatch {
            pi: vec![0; length],
            g: vec![1; length],
            m: vec![None; length],
        }
    }

    fn update(pass: String, m: BaseMatch, l: usize) {
        let pi = estimate_guesses(m, pass);
        if l > 1 {

        }
        let g = factorial(l as u32) * pi;
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


pub fn most_guessable_match_sequence(password: String,
                                     matches: Vec<BaseMatch>,
                                     exclude_additive: bool)
                                     -> PasswordResult {

    let mut optimal: OptimalMatch = OptimalMatch::new(password.len());
    let chars = 0..password.len();

    let matches_by_end = chars.map(|x| matches.iter().filter(|y| y.end == x).collect::<Vec<_>>())
                              .collect::<Vec<_>>();


    for k in 0..password.len() {
        for m in matches_by_end[k].iter() {
            if m.start > 0 {
                // update
            } else {
                // update base case
            }
        }
        // Bruteforce update
    }
    // unwind optimal sequence

    // format result based on length

    PasswordResult { password: password.clone(), ..Default::default() }
}


fn estimate_guesses(m: BaseMatch, password: String) -> u32 {
    0u32
}
