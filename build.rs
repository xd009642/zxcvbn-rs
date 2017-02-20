use std::fs;
use std::io::prelude::*;
use std::collections::HashMap;
use std::error::Error;

//static mut frequencies : HashMap<String, u32> = HashMap::new();
//mut frequencies : Vec<(String, u32)> = vec!();
//static mut list : Vec<String> = vec!();

fn parse_data(data: String) {
    for line in data.lines() {
        let mut l = line.split_whitespace();
        let size = line.split_whitespace().count();
        match size  {
           2=> {
                let freq_string = match l.next() {
                Some(frqs) => frqs,
                None => continue,
                };
                let freq_count = match l.next().unwrap_or("").parse::<u32>(){
                    Ok(f) => println!("{} : {}", freq_string, f),
                    Err(_) => continue,
                };
           },
            //match freq_string.parse::<u32>() {
            //    Ok(f) => frequencies.insert(l.nth(1).unwrap().to_string(), f),
            //    Err(_) => continue,
            //}
         1 => continue,
            //list.push(l.nth(0).unwrap().to_string());
         _ => continue,
        }
    }
}


fn main() {
    // Data files are either lists or frequency tables. Load all files in data
    // and then identify and parse accordingly and generate code
    println!("Generating source from /data/");
    for entry in fs::read_dir("./data").unwrap() {
        let dir = match entry {
            Ok(dir) => dir,
            Err(_) => continue,
        };
        println!("data contains: {:?}", dir.path());
        let mut file = match fs::File::open(dir.path()) {
            Ok(file) => file,
            Err(_) => continue,
        };
        let mut s = String::new();
        match file.read_to_string(&mut s) {
            Err(_) => continue,
            Ok(_) => parse_data(s),
        }
    }

    panic!("Nope");
}
