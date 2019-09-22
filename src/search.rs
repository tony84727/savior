use nbtrs::Tag;
use std::{
    fmt,
    collections::{
        VecDeque,
        HashMap,
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

fn bfs_visit_nbt<C>(compound : &HashMap<String, nbtrs::Tag>, mut callback: C) where C:FnMut(Path,&nbtrs::Tag) {
    let mut worklist = VecDeque::new();
    worklist.push_back((Path::new(), compound));
    while !worklist.is_empty() {
        let (path, compound) = worklist.pop_front().unwrap();
        for (name, tag) in compound.iter() {
            let path = path.concat(name.to_string());
            callback(path.clone(), tag);
            if let Tag::TagCompound(child_compound) = tag {
                worklist.push_back((path, child_compound))
            }
        }
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
                            let tag = match tag {
                                Tag::TagCompound(t) => t,
                                _ => panic!("shoud be a compound"),
                            };
                            bfs_visit_nbt(&tag, |path, tag| {
                                if let Some(value) = self.value {
                                    match tag {
                                        Tag::TagString(s) => {
                                            if s == value {
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
        Ok(None)
    }
}