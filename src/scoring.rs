use result::PasswordResult;
use matching::BaseMatch;


pub fn most_guessable_match_sequence(password: String,
                                     matches: Vec<BaseMatch>,
                                     exclude_additive: bool) -> PasswordResult {
    let chars = 0..password.len();

    let matches_by_end = 
        chars.map(|x| matches.iter().filter(|y| y.end == x).collect::<Vec<_>>())
             .collect::<Vec<_>>();
    
    
    for k in 0..password.len() {
        for m in matches_by_end[k].iter() {
            if m.start > 0 {
                // update
            }else {
                // update base case
            }
        }
        // Bruteforce update
    }
    //unwind optimal sequence
    
    // format result based on length
    
    PasswordResult  { ..Default::default() }
}

