use nbtrs::Tag;
use std::{
    fmt,
    collections::{
        VecDeque,
    },
    time::Instant,
};

struct Keyword {
    string: String,
    integer_8: Option<i8>,
    integer_16: Option<i16>,
    integer_32: Option<i32>,
    float: Option<f32>,
    double: Option<f64>,
}

impl Keyword {
    fn new(keyword: &str) -> Keyword {
        Keyword {
            string: String::from(keyword),
            integer_8: keyword.parse().ok(),
            integer_16: keyword.parse().ok(),
            integer_32: keyword.parse().ok(),
            float: keyword.parse().ok(),
            double: keyword.parse().ok(),
        }
    }
}

pub struct Searcher {
    key: Option<Keyword>,
    value: Option<Keyword>,
}

pub struct Path(Vec<String>);

impl Clone for Path {
    fn clone(&self) -> Self {
        Path(self.0.clone())
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.join("."))
    }
}

impl Path {
    fn new() -> Self {
        Path(Vec::new())
    }
    fn concat(&self, path_segment: String) -> Self {
        let mut new = self.0.clone();
        new.push(path_segment);
        Path(new)
    }
}

pub enum MatchedValue {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
}

pub enum Matched {
    Key(String),
    Value(MatchedValue),
}

pub struct SearchResult {
    pub path: Path,
    pub matched: Matched,
}

fn bfs_visit_nbt<C>(start : &nbtrs::Tag, mut callback: C) where C:FnMut(Path,&nbtrs::Tag) {
    let mut worklist = VecDeque::new();
    worklist.push_back((Path::new(), start));
    while !worklist.is_empty() {
        let (path,tag) = worklist.pop_front().unwrap();
        match tag {
            Tag::TagCompound(compound) => {
                for (name, tag) in compound.iter() {
                    let child_path = path.concat(name.to_string());
                    worklist.push_back((child_path, tag));
                }
            },
            Tag::TagList(list) => {
                for (i,tag) in list.iter().enumerate() {
                    let child_path = path.concat(i.to_string());
                    worklist.push_back((child_path, tag));
                }
            },
            _ =>(),
        }
        callback(path, tag);
    }
}

#[derive(Debug)]
pub enum SearcherError {
    // throwed when both key/value are None
    IllegalArguments,
    NBT(nbtrs::Error),
}

impl Searcher {
    pub fn new(key: Option<&str>, value: Option<&str>) -> Searcher {
        Searcher{
            key: key.map(Keyword::new),
            value: value.map(Keyword::new),
        }
    }
    pub fn search<R>(&self, file: &mut nbtrs::RegionFile<R>)-> Result<Option<Vec<SearchResult>>, SearcherError>
    where R: std::io::Seek + std::io::Read  {
        if self.key.is_none() && self.value.is_none() {
            return Err(SearcherError::IllegalArguments)
        }
        let mut matches = Vec::new();
        let start = Instant::now();
        let mut scan_counter = 0;
        for x in 0..=31 {
            for z in 0..=31 {
                if file.chunk_exists(x,z) {
                    match file.load_chunk(x,z) {
                        Ok(tag) => {
                            bfs_visit_nbt(&tag, |path, tag| {
                                scan_counter = scan_counter + 1;
                                if let Some(value) = &self.value {
                                    match tag {
                                        Tag::TagString(s) => {
                                            if s.contains(&value.string[..]) {
                                                matches.push(SearchResult{
                                                    path: path,
                                                    matched: Matched::Value(MatchedValue::String(s.to_string()))
                                                })
                                            }
                                        },
                                        _ => (),
                                    }
                                }
                            })
                        },
                        Err(err) => return Err(SearcherError::NBT(err)),
                    }
                }
            }
        }
        let duration = Instant::now().duration_since(start);
        println!("search {} entries, in: {:?} {} entries/s", scan_counter, duration, f64::from(scan_counter) / duration.as_secs_f64());
        if matches.len() > 0 {
            Ok(Some(matches))
        } else {
            Ok(None)
        }
    }
}