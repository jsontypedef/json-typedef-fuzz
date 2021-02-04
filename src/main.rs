use anyhow::{Context, Result};
use clap::{crate_version, App, AppSettings, Arg};
use jtd::Schema;
use rand::SeedableRng;
use rand_pcg::Pcg32;

use std::fs::File;
use std::io::{stdin, BufReader, Read};

fn main() -> Result<()> {
    let matches = App::new("jtd-fuzz")
        .version(crate_version!())
        .about("Generate random JSON documents from a given JSON Typedef schema")
        .setting(AppSettings::ColoredHelp)
        .arg(
            Arg::with_name("num-values")
                .help("How many values to generate.")
                .short("n")
                .long("num-values")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("seed")
                .help("Random number generator seed.")
                .short("s")
                .long("seed")
                .takes_value(true),
        )
        .arg(Arg::with_name("file").help("Read input from this file, instead of STDIN"))
        .get_matches();

    // Parse num-values and seed first, so that we can give the user an error
    // before potentially blocking as we read in the schema.

    let mut rng = if let Some(seed) = matches.value_of("seed") {
        Pcg32::seed_from_u64(
            seed.parse()
                .with_context(|| format!("Failed to parse seed: {}", seed))?,
        )
    } else {
        Pcg32::from_entropy()
    };

    let num_values: Option<u64> = if let Some(n) = matches.value_of("num-values") {
        Some(
            n.parse()
                .with_context(|| format!("Failed to parse number of values: {}", n))?,
        )
    } else {
        None
    };

    let input: Box<dyn Read> = if let Some(file) = matches.value_of("file") {
        Box::new(BufReader::new(File::open(file)?))
    } else {
        Box::new(stdin())
    };

    let schema = Schema::from_serde_schema(
        serde_json::from_reader(input).with_context(|| "Failed to parse schema")?,
    )
    .with_context(|| "Malformed schema")?;

    schema.validate().with_context(|| "Invalid schema")?;

    // let serde_schema: SerdeSchema =
    //     serde_json::from_reader(input).with_context(|| format!("Failed to parse schema"))?;

    // let schema: Schema = serde_schema
    //     .try_into()
    //     .map_err(|err| format_err!("invalid schema: {:?}", err))
    //     .with_context(|| format!("Failed to load schema"))?;

    // schema
    //     .validate()
    //     .map_err(|err| format_err!("invalid schema: {:?}", err))
    //     .with_context(|| format!("Failed to validate schema"))?;

    if let Some(n) = num_values {
        for _ in 0..n {
            println!("{}", jtd_fuzz::fuzz(&schema, &mut rng));
        }
    } else {
        loop {
            println!("{}", jtd_fuzz::fuzz(&schema, &mut rng));
        }
    }

    Ok(())
}
