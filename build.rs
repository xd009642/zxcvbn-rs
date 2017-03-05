#[macro_use]
extern crate slog;
extern crate slog_stream;
extern crate slog_stdlog;
#[macro_use]
extern crate log;

use std::env;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::io::Write;
use std::path::Path;
use std::collections::HashMap;
use std::cell::RefCell;
use std::error::Error;

use slog::DrainExt;

enum KeyAlignment {
    Slanted,
    Aligned,
}

struct WordData {
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
        }
    }
}

/// Use trailing spaces at the start of lines to represent non character keys
/// such as caps tab and shift.
fn generate_adjacencies(keyboard: String, 
                        align: KeyAlignment) -> HashMap<String, String> {
    let mut adj_list : HashMap<String, String> = HashMap::new();
    let rows = keyboard.lines()
        .map(|x| x.split(' ').collect::<Vec<&str>>())
        .collect::<Vec<Vec<&str>>>();
    for (i, row) in rows.iter().enumerate() {
        for (j, key) in row.iter().enumerate() {
            if !key.is_empty() {
                let mut value : String = String::new();
                // Values for row above
                if i>0 {
                    let prev = rows.get(i-1).unwrap();
                    if let Some(pchar) = prev.get(j) {
                        let ap = format!("{} ", pchar);
                        value.push_str(ap.as_str());
                    }
                    if let Some(pchar) = prev.get(j+1) {
                        let ap = format!("{} ", pchar);
                        value.push_str(ap.as_str());
                    }
                }
                // Values for this row
                if j > 0 {
                    let ap = format!("{} ", row.get(j-1).unwrap());
                    value.push_str(ap.as_str());
                }
                if let Some(next) = row.get(j+1) {
                    let ap = format!("{} ", next);
                    value.push_str(ap.as_str());
                }
                // Values for next row
                if let Some(next) = rows.get(i+1) {
                    if let Some(nchar) = next.get(j) {
                        if !nchar.is_empty()
                        {
                            let ap = format!("{} ", nchar);
                            value.push_str(ap.as_str());
    
                            if let Some(nchar) = next.get(j+1) {
                                let ap = format!("{} ", nchar);
                                value.push_str(ap.as_str());
                            }
                        }
                    } 
                }
                adj_list.insert(key.to_string(), value);
            }
        }
    }
    adj_list
}

fn export_adjacencies(target: &mut fs::File, name: String, 
                      data: HashMap<String, String>) {
    let mut source:String = String::new();
    let header = format!("\tstatic ref {}:HashMap<&'static str, &'static str> = {{\n\
    \t\tlet mut m = HashMap::new();\n", name);

    source.push_str(header.as_str());

    for (k, v) in &data {
        let line = format!("\t\tm.insert(\"{}\", \"{}\");\n", k, v);
        source.push_str(line.as_str());
    }
    source.push_str("\t\tm\n\t};\n");
    if let Err(e) = target.write_all(source.as_bytes()) {
        error!("Failed to export {} : {}", name, e.description());
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

    // Trailing space after \n is to represent offset of keyboard. (¬ is hard)
    let qwerty_uk = "¬` 1! 2\\\" 3£ 4$ %5 6^ 7& 8* 9( 0) -_ =+\n \
                    qQ wW eE rR tT yY uU iI oO pP [{ ]}\n \
                    aA sS dD fF gG hH jJ kK lL ;: '@ #~\n \
                    \\\\| zZ xX cC vV bB nN mM ,< /?".to_string();

    let dvorak = "~ 1! 2@ 3# 4$ 5% 6^ 7& 8* 9( 0) [{ ]}\n \
                  '\\\" ,< .> pP yY fF gG cC rR lL /? =+ \\\\|\n \
                  aA oO eE uU iI dD hH tT nN sS -_\n \
                  ;: qQ jJ kK xX bB mM vV zZ".to_string();

    println!("Generating keyboard adjacency lookups");
    let dest_path = Path::new(&out_dir).join("adjacency_data.rs");
    let mut f = fs::File::create(&dest_path).unwrap();

    f.write_all(b"//AUTOGENERATED DO NOT EDIT\n\
    use std::collections::HashMap;\n\
    \n\
    lazy_static! {\n").unwrap();

    if let Ok(mut clone_file) = f.try_clone() {
        let data = generate_adjacencies(qwerty_uk, KeyAlignment::Slanted);
        export_adjacencies(&mut clone_file, "QWERTY".to_string(), data);
    }
    if let Ok(mut clone_file) = f.try_clone() {
        let data = generate_adjacencies(dvorak, KeyAlignment::Slanted);
        export_adjacencies(&mut clone_file, "DVORAK".to_string(), data);
    }

    f.write_all(b"}\n").unwrap();

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
