#[macro_use]
extern crate slog;
extern crate slog_stream;
extern crate slog_stdlog;
#[macro_use]
extern crate log;
extern crate num_traits;

use std::env;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::io::Write;
use std::path::Path;
use std::collections::{HashMap, HashSet};
use num_traits::checked_pow;
use std::cell::RefCell;
use std::error::Error;

use slog::DrainExt;


struct WordData {
    name: String,
    data: RefCell<Vec<String>>,
}


fn parse_data(data: String) -> Vec<String> {
    let mut word_list : Vec<String> = Vec::new();
    let mut checker: HashSet<String> = HashSet::new();
    for line in data.lines() {
        let mut l = line.split_whitespace();
        let size = line.split_whitespace().count();
        match size  {
            1 | 2=> {
                match l.next() {
                    Some(frqs) => {
                        if checker.insert(frqs.to_string()) {
                            word_list.push(frqs.to_string());
                        }
                    },
                    None => continue,
                };  
            },
            _ => continue,
        }
    }
    word_list
}

fn is_rare_and_short(word: &String, rank: u32) -> bool {
    let len = word.chars().count();
    rank >= checked_pow(10u32, len).unwrap_or(u32::max_value())
}

fn filter_data(dicts: &mut Vec<WordData>) {
    let mut best_match : HashMap<String, (usize, String)> = HashMap::new();
    // Build best matches. Shows precedence of words in different dictionaries
    for datum in dicts.iter() {
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

    for datum in dicts {    
        let conditional = |w: &String| {
            best_match.get(w).unwrap().1 == datum.name && 
                !is_rare_and_short(w, best_match.get(w).unwrap().0 as u32)
        };

        let mut words = datum.data.borrow_mut();
        words.retain(conditional);
    }
}


fn main() {

    let build_log = "build.log";
    let log_file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(build_log).unwrap();
    let drain = slog_stream::stream(log_file, LogFormat).fuse();
    let logger = slog::Logger::root(drain, o!());
    slog_stdlog::set_logger(logger).unwrap();

    info!("Building zxcvbn_rs.");
    
    // Data files are either lists or frequency tables. Load all files in data
    // and then identify and parse accordingly and generate code
    info!("Generating source from /data/");
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
                let temp = WordData{ 
                    name: name.to_string(), 
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
        if let Some(limit) = limits.get(lists.name.as_str()) {
            lists.data.borrow_mut().truncate(limit.clone());
        }
    }
    info!("Exporting frequency based data");
    let mut source : String = String::new();
    
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("frequency_data.rs");
    let mut f = fs::File::create(&dest_path).unwrap();
    
    for lists in exported_data.iter() {
        let mut line = format!("static {}: &'static [&'static str] = &[ \n", 
                               lists.name.to_uppercase());
        
        let data = lists.data.borrow();

        for word in data.iter() {
            let entry = format!("\t\"{}\",\n", word);
            line.push_str(entry.as_str());
        }
        line.push_str("];\n\n");
        source.push_str(line.as_str());
    }
    match f.write_all(source.as_bytes()) {
        Ok(_) => info!("Successfully exported frequency data"),
        Err(e) => error!("{}", e.description()),
    }
    f.sync_all().unwrap();

    info!("Code generation finished");
}


struct LogFormat;

impl slog_stream::Format for LogFormat {
    fn format(&self,
              io: &mut io::Write,
              rinfo: &slog::Record,
              _logger_values: &slog::OwnedKeyValueList)
        -> io::Result<()> {
            let msg = format!("{} - {}\n", rinfo.level(), rinfo.msg());
            let _ = try!(io.write_all(msg.as_bytes()));
            Ok(())
        }

}
