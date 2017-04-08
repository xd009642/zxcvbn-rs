extern crate zxcvbn_rs;

use zxcvbn_rs::{matching, scoring, result};
use std::env;

fn zxcvbn(password: String, user_dictionary: Vec<String>) {
    println!("Password is {}", password);
    let matches = matching::omnimatch(password.as_ref());
    let mut best_sequence = scoring::most_guessable_match_sequence(password, 
                                                                   matches, 
                                                                   false);
    let attack_times = result::CrackTimes::new(best_sequence.guesses);
    best_sequence.get_feedback();
    println!("{:?}", best_sequence);
}

fn main() {
    let password: Option<String> = env::args().nth(1);
    let user_dictionary: Vec<String> = env::args().skip(2).collect();
    match password {
        Some(x) => zxcvbn(x, user_dictionary), 
        None => println!("Must provide a password"),
    }
}
