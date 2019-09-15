use nbtrs::Tag;
use std::convert::TryFrom;

pub struct Info {
    pub chunk_count: u16,
    pub tile_entity_count: u64,
    pub entity_count: u64,
    // chunks that failed to load
    pub corrupted_count: u16,
}

pub fn inspect<R>(file: R) -> Result<Info, nbtrs::Error>
where R: std::io::Read + std::io::Seek
{
    nbtrs::RegionFile::new(file).map(|mut region|{
        let mut info = Info{
            chunk_count: 0,
            tile_entity_count: 0,
            entity_count: 0,
            corrupted_count: 0,
        };
        for x in 0..=31 {
            for z in 0..=31 {
                if region.chunk_exists(x,z) {
                    info.chunk_count = info.chunk_count + 1;
                    match region.load_chunk(x,z) {
                        Ok(tag) => {
                            match tag {
                                Tag::TagCompound(compound) => {
                                    for (name, tag) in compound.iter() {
                                        match &name[..] {
                                            "Level" => {
                                                if let Tag::TagCompound(level) = tag {
                                                    for (name, tag) in level.iter() {
                                                        match &name[..] {
                                                            "Entities" => {
                                                                if let Tag::TagList(entity_list) = tag {
                                                                    info.entity_count = info.entity_count + u64::try_from(entity_list.len()).unwrap();
                                                                }
                                                            },
                                                            "TileEntities" => {
                                                                if let Tag::TagList(tile_entity_list) = tag {
                                                                    info.tile_entity_count = info.tile_entity_count + u64::try_from(tile_entity_list.len()).unwrap();
                                                                }
                                                            }
                                                            _ => (),
                                                        }
                                                    }
                                                }
                                            },
                                            _ => (),
                                        }
                                    }
                                }
                                _ => ()
                            }
                        },
                        Err(err) => {
                            println!("DEBUG: {}", err);
                            info.corrupted_count = info.corrupted_count + 1;
                        }
                    }
                }
            }
        }
        info
    })
}