use std::fs;
use std::io::prelude::*;
use std::collections::HashMap;



fn parse_data(data: String, word_list: &mut Vec<String>, 
              freqs: &mut HashMap<String, u32>) {
    for line in data.lines() {
        let mut l = line.split_whitespace();
        let size = line.split_whitespace().count();
        match size  {
            2=> {
                let freq_string = match l.next() {
                Some(frqs) => frqs.to_string(),
                None => continue,
                };
                let count = match l.next().unwrap_or("").parse::<u32>(){
                    Ok(f) => f,
                    Err(_) => continue,
                };
                freqs.insert(freq_string, count);
            },
            1 => word_list.push(l.nth(0).unwrap().to_string()),
            _ => continue,
        }
    }
}


fn main() {
    // Data files are either lists or frequency tables. Load all files in data
    // and then identify and parse accordingly and generate code
    println!("Generating source from /data/");
    let mut words : Vec<String> = Vec::new();
    let mut frequencies : HashMap<String, u32> = HashMap::new();

    for entry in fs::read_dir("./data").unwrap() {
        let dir = match entry {
            Ok(dir) => dir,
            Err(_) => continue,
        };
        let mut file = match fs::File::open(dir.path()) {
            Ok(file) => file,
            Err(_) => continue,
        };
        let mut s = String::new();
        match file.read_to_string(&mut s) {
            Err(_) => continue,
            Ok(_) => parse_data(s, &mut words, &mut frequencies),
        }
    }
    panic!("Nope");
}
