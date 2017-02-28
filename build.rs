use std::env;
use std::fs;
use std::io::prelude::*;
use std::io::Write;
use std::path::Path;
use std::collections::HashMap;
use std::cell::RefCell;

struct WordData
{
    name: String,
    count: Option<usize>,
    data: RefCell<Vec<String>>,
}


fn parse_data(data: String) -> Vec<String> {
    let mut word_list : Vec<String> = Vec::new();

    for line in data.lines() {
        let mut l = line.split_whitespace();
        let size = line.split_whitespace().count();
        match size  {
            1 | 2=> {
                match l.next() {
                    Some(frqs) => word_list.push(frqs.to_string()),
                    None => continue,
                };  
            },
            _ => continue,
        }
    }
    word_list
}

fn is_rare_and_short(word: &String, rank: u32) -> bool {
    let result = if word.len() < 8 {
        rank >=10u32.pow(word.len() as u32)
    } else { 
        false
    };
    result
}

fn filter_data(dicts: &mut Vec<WordData>) {
    let mut best_match : HashMap<String, (usize, String)> = HashMap::new();
    // Build best matches. Shows precedence of words in different dictionaries
    for datum in dicts.iter() {
        if datum.count.is_some() {
            let words = datum.data.borrow();
            for (rank, word) in words.iter().enumerate() {
                if best_match.contains_key(word) {
                    if rank < best_match.get(word).unwrap().0 {
                        best_match.insert(word.clone(), 
                                          (rank, datum.name.clone()));
                    } 
                } else {
                    best_match.insert(word.clone(), 
                                      (rank, datum.name.clone()));
                }
            }
        }
    }

    for datum in dicts {
        if datum.count.is_some() {
        
            let conditional = |w: &String| { 
                best_match.get(w).unwrap().1 == datum.name && 
                    !is_rare_and_short(w, best_match.get(w).unwrap().0 as u32)
            };

            let mut words = datum.data.borrow_mut();
            words.retain(conditional);
            /*for word in words.iter_mut() {
                if best_match.get(word).unwrap().1 != datum.name {
                    
                }
            }*/
        }
    }
}

// Choose word limits for each file...
fn main() {
    // Data files are either lists or frequency tables. Load all files in data
    // and then identify and parse accordingly and generate code
    println!("Generating source from /data/");
    let limits : HashMap<&str, usize> = {
        let mut map = HashMap::new();
        map.insert("us_tv_and_film", 30000);
        map.insert("english_wikipedia", 30000);
        map.insert("passwords", 30000);
        map.insert("surnames", 10000);
        map
    };


    let mut exported_data : Vec<WordData> = Vec::new();
    
    for entry in fs::read_dir("./data").unwrap() {
        let dir = match entry {
            Ok(dir) => dir,
            Err(_) => continue,
        };
        let path = dir.path();
        let file_name = path.file_stem();
        let mut file = match fs::File::open(dir.path()) {
            Ok(file) => file,
            Err(_) => continue,
        };
        let mut s = String::new();
        
        match file.read_to_string(&mut s) {
            Err(_) => continue,
            Ok(_) => { 
                let name = match file_name.unwrap().to_str() {
                    Some(fname) => fname,
                    None => continue,
                };
                let count = match limits.get(name) {
                    Some(t) => Some(*t),
                    None => None,
                };
                let temp = WordData{ name: name.to_string(), count: count,
                                          data: RefCell::new(parse_data(s)),
                };
                exported_data.push(temp);
            },
        }
    }
    println!("Filtering data");
    filter_data(&mut exported_data);
    println!("Applying size limits");
    // Apply limits
    for lists in exported_data.iter_mut() {
        println!("Looking for limits for {}", lists.name);
        if let Some(limit) = limits.get(lists.name.as_str()) {
            println!("Limit is {}", limit);
            lists.data.borrow_mut().truncate(limit.clone());
        }
    }
    println!("Exporting data");
    let mut source : String = String::new();
    
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("global_data.rs");
    let mut f = fs::File::create(&dest_path).unwrap();
    
    for lists in exported_data.iter() {
        let mut line = format!("static {}: &'static [String] = &[ \n", 
                               lists.name);
        
        let data = lists.data.borrow();

        for word in data.iter() {
            let entry = format!("\t\"{}\",\n", word);
            line.push_str(entry.as_str());
        }
        line.push_str("];\n\n");
        source.push_str(line.as_str());
    }


    f.write_all(source.as_bytes()).unwrap();
    panic!("Test");
}
