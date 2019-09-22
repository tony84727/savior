use nbtrs::Tag;
use std::fmt;

pub struct Searcher<'a> {
    pub key: Option<&'a str>,
    pub value: Option<&'a str>,
}

struct Path {
    segments: Vec<String>,
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.segments.join("."))
    }
}

impl Path {
    fn concat(&self, path_segment: String) -> Self {
        let mut new = self.segments.clone();
        new.push(path_segment);
        Path{
            segments: new,
        }
    }
}

pub struct SearchResult {
    pub path: Path,
}

#[derive(Debug)]
pub enum SearcherError {
    // throwed when both key/value are None
    IllegalArguments,
    NBT(nbtrs::Error),
}

impl<'a> Searcher<'a> {
    fn visit(&self, parent_path: Vec<String>, tag: &nbtrs::Tag) -> Option<Vec<SearchResult>> {
        let mut search_results = Vec::new();
        match tag {
            Tag::TagString(s) => {
                if let Some(value) = self.value {
                    if s == value {
                        search_results.push(SearchResult{
                            path: Path{
                                segments: parent_path,
                            }
                        })
                    }
                }
            }
            _ => ()
        }
        if search_results.len() <= 0 {
            return None
        }
        Some(search_results)
    }

    pub fn search<R>(&self, file: &mut nbtrs::RegionFile<R>)-> Result<Option<Vec<SearchResult>>, SearcherError>
    where R: std::io::Seek + std::io::Read  {
        if self.key == None && self.value == None {
            return Err(SearcherError::IllegalArguments)
        }
        for x in 0..=31 {
            for z in 0..=31 {
                if file.chunk_exists(x,z) {
                    match file.load_chunk(x,z) {
                        Ok(tag) => {
                            self.visit(Vec::new(), tag: &nbtrs::Tag)
                        },
                        Err(err) => return Err(SearcherError::NBT(err)),
                    }
                }
            }
        }
        Ok(None)
    }
}