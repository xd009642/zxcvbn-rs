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

fn filter_data(dicts: &mut Vec<WordData>) {
    println!("Not yet implemented");
    for datum in dicts {
        if datum.count.is_some() {
            let mut words = datum.data.borrow_mut();
            let count = datum.count.unwrap();
            if words.len() > count {
                words.resize(count, String::new());
            }
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

    filter_data(&mut exported_data);
    println!("Size of exported data is {}", exported_data.len());
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
