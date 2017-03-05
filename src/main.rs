extern crate zxcvbn_rs;

use zxcvbn_rs::{matching};
use std::env;

fn zxcvbn(password: String, user_dictionary: Vec<String>) {
    println!("Password is {}", password);
    let matches = matching::omnimatch(password.as_ref());
    println!("User provided custom dict: {:?}", user_dictionary);
}

fn main() {
    let password: Option<String> = env::args().nth(1);
    let user_dictionary: Vec<String> = env::args().skip(2).collect();
    match password {
        Some(x) => zxcvbn(x, user_dictionary), 
        None => println!("Must provide a password"),
    }
}
