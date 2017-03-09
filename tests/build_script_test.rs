include!(concat!(env!("OUT_DIR"), "/frequency_data.rs"));

use std::fs;
use std::path::Path;
use std::io;
use std::io::Read;

fn get_words(data: String) -> Vec<String> {
    let mut word_list : Vec<String> = Vec::new();

    for line in data.lines() {
        let mut l = line.split_whitespace();
        let size = line.split_whitespace().count();
        match size {
            1 | 2 => {
                match l.next() {
                    Some(w) => word_list.push(w.to_string()),
                    None => continue,
                };
            },
            _ => continue,
        }
    }
    word_list
}

#[test]
fn no_duplicates() {
    let mut words:Vec<String> = Vec::new();
    for entry in fs::read_dir("./data").unwrap() {
        let dir = match entry {
            Ok(directory) => directory,
            Err(_) => continue,
        };
        let path = dir.path();
        let file_name = path.file_stem();
        let mut file = match fs::File::open(dir.path()) {
            Ok(file) => file,
            Err(_) => continue,
        };

        let mut s = String::new();

        if file.read_to_string(&mut s).is_ok() {
            let name = match file_name.unwrap().to_str() {
                Some(fname) => fname,
                None => continue,
            };
            words.append(&mut get_words(s));
        }
    }
    words.sort();
    words.dedup();
    
    let dicts = vec![FEMALE_NAMES, MALE_NAMES, SURNAMES, PASSWORDS,
                     ENGLISH_WIKIPEDIA, US_TV_AND_FILM];
    let dict_names=vec!["Female names", "Male names", "Surnames", "Passwords",
                        "Wikipedia", "TV and film"];
    
    for word in words {
        let mut appearances:Vec<&str> = Vec::new();
        let mut count:usize=0;
        for (i, d) in dicts.iter().enumerate() {
            if d.to_vec().contains(&word.as_ref()) {
                count += 1;
                appearances.push(dict_names[i]);
            }
        }
        assert!(count<2, "Failed: {} appears {} times\n\
                Appears in: {:?}", word, count, appearances);
    }
}
