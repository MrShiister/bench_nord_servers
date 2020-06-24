use bench_nord_servers::{run, Config};
use std::{env, process, error::Error};

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem with parsing arguments: {}", err);
        process::exit(1);
    });

    run(config).unwrap_or_else(|err| {
        eprintln!("Error running program: {}", err);
        process::exit(1);
    });

    Ok(())

}
