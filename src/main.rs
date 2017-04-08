extern crate zxcvbn_rs;

use zxcvbn_rs::{matching, scoring, result};
use std::env;

fn zxcvbn(password: String, user_dictionary: Vec<String>) {
    println!("Password is {}", password);
    let matches = matching::omnimatch(password.as_ref());
    let best_sequence = scoring::most_guessable_match_sequence(password, matches, false);
    let attack_times = result::CrackTimes::new(best_sequence.guesses);
    let feedback = result::get_feedback(best_sequence.guesses);
    println!("{:?}", feedback);
    println!("Guesses are: {}", best_sequence.guesses);
    println!("Attack times in seconds\n{:?}", attack_times);
}

fn main() {
    let password: Option<String> = env::args().nth(1);
    let user_dictionary: Vec<String> = env::args().skip(2).collect();
    match password {
        Some(x) => zxcvbn(x, user_dictionary), 
        None => println!("Must provide a password"),
    }
}
