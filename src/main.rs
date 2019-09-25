mod inspect;
mod search;
fn new_subcommand(name: &'_ str) -> clap::App<'_, '_> {
    clap::SubCommand::with_name(name)
    .author("tony84727 <tony84727@gmail.com>")
}
fn main() {
    let matches = clap::App::new("savior")
        .about("Let's save the corrupted minecraft world save together!")
        .author("tony84727 <tony84727@gmail.com>")
        .subcommand(
            new_subcommand("inspect")
            .about("Inspect region file")
            .arg(
                clap::Arg::with_name("target")
                .required(true)
                .index(1)
                .long_help("Target to inspect, should be a region file(.mca)
inspecting world directory/level.dat is WIP")
            )
        )
        .subcommand(
            new_subcommand("search")
            .about("search keyword in the NBT tree")
            .arg(clap::Arg::with_name("target").index(1).required(true).help("Region file to search"))
            .arg(clap::Arg::with_name("key").short("k").takes_value(true).help("Keyword for nbt name"))
            .arg(clap::Arg::with_name("value").short("v").takes_value(true).help("Keyword for nbt value"))
        )
        .get_matches();
    if let Some(matches) = matches.subcommand_matches("inspect") {
        match matches.value_of("target") {
            None => {
                println!("target required")
            },
            Some(target) => {
                println!("inspecting {} ....", target);
                match std::fs::File::open(target) {
                    Err(err) => {
                        println!("{}", err);
                    },
                    Ok(file) => {
                        match inspect::inspect(file) {
                            Ok(info) => {
                                println!("number of chunks: {}", info.chunk_count);
                                println!("number of entities: {}", info.entity_count);
                                println!("number of tile entities: {}", info.tile_entity_count);
                            }
                            Err(err) => {
                                println!("nbt parsing failed, error: {}", err)
                            }
                        }
                    }
                }
            }
        };
    }

    if let Some(matches) = matches.subcommand_matches("search") {
        let key = matches.value_of("key");
        let value = matches.value_of("value");
        if key == None && value == None {
            println!("must specify either key or value to search");
        } else {
            match std::fs::File::open(matches.value_of("target").unwrap()) {
                Err(err) => {
                    println!("{}", err);
                },
                Ok(file) => {
                    match nbtrs::RegionFile::new(file) {
                        Err(err) => {
                            println!("fail to parse the region file, error: {}", err)
                        },
                        Ok(mut region) => {
                            let searcher = search::Searcher::new(key, value);
                            match searcher.search(&mut region) {
                                Err(err) => {
                                    println!("{:?}", err);
                                },
                                Ok(results) => match results {
                                    None => println!("no match found"),
                                    Some(results) => {
                                        println!("{} matches:", results.len());
                                        for result in results {
                                            println!("{}", result.path)
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
        }
    }
}
