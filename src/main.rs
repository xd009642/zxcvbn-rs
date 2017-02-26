extern crate zxcvbn_rs;

use std::env;
include!(concat!(env!("OUT_DIR"), "/global_data.rs"));

fn zxcvbn(password: String, user_dictionary: Vec<String>) {
    println!("Password is {}", password);
    println!("User provided custom dict: {:?}", user_dictionary);
}

fn main() {
    let password : Option<String> = env::args().nth(1);
    let user_dictionary: Vec<String> = env::args().skip(2).collect();
    match password
    {
        Some(x) => zxcvbn(x, user_dictionary), 
        None => println!("Must provide a password"),
    }
    for value in frequencies.iter() {
        println!("{}", value);
    }
}
