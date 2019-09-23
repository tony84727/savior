use nbtrs::Tag;
use std::{
    fmt,
    collections::{
        VecDeque,
    }
};

pub struct Searcher<'a> {
    pub key: Option<&'a str>,
    pub value: Option<&'a str>,
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

pub struct SearchResult {
    pub path: Path,
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

impl<'a> Searcher<'a> {
    pub fn search<R>(&self, file: &mut nbtrs::RegionFile<R>)-> Result<Option<Vec<SearchResult>>, SearcherError>
    where R: std::io::Seek + std::io::Read  {
        if self.key == None && self.value == None {
            return Err(SearcherError::IllegalArguments)
        }
        let mut matches = Vec::new();
        for x in 0..=31 {
            for z in 0..=31 {
                if file.chunk_exists(x,z) {
                    match file.load_chunk(x,z) {
                        Ok(tag) => {
                            bfs_visit_nbt(&tag, |path, tag| {
                                if let Some(value) = self.value {
                                    match tag {
                                        Tag::TagString(s) => {
                                            if s.contains(value) {
                                                matches.push(SearchResult{
                                                    path: path,
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
        if matches.len() > 0 {
            Ok(Some(matches))
        } else {
            Ok(None)
        }
    }
}