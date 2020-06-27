use bench_nord_servers::{run, Config};
use std::{env, process, time::Instant, error::Error};

fn main() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();

    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem with parsing arguments: {}", err);
        process::exit(1);
    });

    run(config).unwrap_or_else(|err| {
        eprintln!("Error running program: {}", err);
        process::exit(1);
    });

    let duration = start.elapsed();

    println!("Time elapsed: {:?}", duration);

    Ok(())

}
