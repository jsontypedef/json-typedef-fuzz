use anyhow::{Context, Result};
use clap::{load_yaml, crate_version, App, AppSettings};
use jtd::Schema;
use rand::SeedableRng;
use rand_pcg::Pcg32;

use std::fs::File;
use std::io::{stdin, BufReader, Read};

fn main() -> Result<()> {
    let cli_yaml = load_yaml!("cli.yaml");
    let matches = App::from(cli_yaml)
        .setting(AppSettings::ColoredHelp)
        .version(crate_version!())
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

    let reader = BufReader::new(match matches.value_of("input").unwrap() {
        "-" => Box::new(stdin()) as Box<dyn Read>,
        file @ _ => Box::new(File::open(file)?) as Box<dyn Read>,
    });

    let schema = Schema::from_serde_schema(
        serde_json::from_reader(reader).with_context(|| "Failed to parse schema")?,
    )
    .with_context(|| "Malformed schema")?;

    schema.validate().with_context(|| "Invalid schema")?;

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
