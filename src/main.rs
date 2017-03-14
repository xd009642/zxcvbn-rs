extern crate zxcvbn_rs;

use zxcvbn_rs::{matching, scoring};
use std::env;

fn zxcvbn(password: String, user_dictionary: Vec<String>) {
    println!("Password is {}", password);
    let matches = matching::omnimatch(password.as_ref());
    println!("Found {} matches \n{:?}\n",matches.len(), matches);
    let best_sequence = scoring::most_guessable_match_sequence(password,
                                                               matches, 
                                                               false);

    println!("Guesses are: {}", best_sequence.guesses);
}

fn main() {
    let password: Option<String> = env::args().nth(1);
    let user_dictionary: Vec<String> = env::args().skip(2).collect();
    match password {
        Some(x) => zxcvbn(x, user_dictionary), 
        None => println!("Must provide a password"),
    }
}
