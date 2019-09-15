mod inspect;
fn main() {
    let matches = clap::App::new("savior")
        .about("Let's save the corrupted minecraft world save together!")
        .author("tony84727 <tony84727@gmail.com>")
        .subcommand(
            clap::SubCommand::with_name("inspect")
            .help("inspect region file")
            .arg(
                clap::Arg::with_name("target")
                .required(true)
                .index(1)
                .long("target to inspect, should be a region file(.mca)\ninspecting world directory/level.dat is WIP")
            )
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
}
