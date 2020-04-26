use chrono::{DateTime, NaiveDateTime, Utc};
use clap::{crate_version, App, AppSettings, Arg};
use failure::{format_err, Error};
use jtd::{form, Form, Schema, SerdeSchema};
use rand::rngs::SmallRng;
use rand::seq::IteratorRandom;
use rand::SeedableRng;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::fs::File;
use std::io;

fn main() -> Result<(), Error> {
    let matches = App::new("jtd-fuzz")
        .version(crate_version!())
        .about("Creates random JSON documents satisfying a JSON Type Definition schema")
        .setting(AppSettings::ColoredHelp)
        .arg(
            Arg::with_name("n")
                .help("How many values to generate. Zero (0) indicates infinity")
                .default_value("0")
                .short("n")
                .long("num-values"),
        )
        .arg(
            Arg::with_name("s")
                .help("A seed for the random number generator used internally. Zero (0) disables seeding, and uses an entropy source instead")
                .default_value("0")
                .short("s")
                .long("seed"),
        )
        .arg(
            Arg::with_name("INPUT")
                .help("Where to read schema from. Dash (hypen) indicates stdin")
                .default_value("-"),
        )
        .get_matches();

    let num_values: usize = matches.value_of("n").unwrap().parse()?;

    let reader: Box<dyn io::Read> = match matches.value_of("INPUT").unwrap() {
        "-" => Box::new(io::stdin()),
        file @ _ => Box::new(io::BufReader::new(File::open(file)?)),
    };

    let serde_schema: SerdeSchema = serde_json::from_reader(reader)?;
    let schema: Schema = serde_schema
        .try_into()
        .map_err(|err| format_err!("{:?}", err))?;

    let mut rng = match matches.value_of("s").unwrap().parse::<u64>()? {
        0 => SmallRng::from_entropy(),
        n @ _ => SmallRng::seed_from_u64(n),
    };

    let mut i = 0;
    while i != num_values || num_values == 0 {
        println!("{}", fuzz(&schema, &mut rng, &schema));
        i += 1;
    }

    Ok(())
}

fn fuzz<R: rand::Rng + ?Sized>(root: &Schema, rng: &mut R, schema: &Schema) -> Value {
    match schema.form {
        Form::Empty => fuzz_any(root, rng),
        Form::Ref(form::Ref {
            ref definition,
            nullable,
        }) => {
            if nullable && rng.gen() {
                Value::Null
            } else {
                fuzz(root, rng, root.definitions.get(definition).unwrap())
            }
        }
        Form::Type(form::Type {
            ref type_value,
            nullable,
        }) => {
            if nullable && rng.gen() {
                Value::Null
            } else {
                match type_value {
                    form::TypeValue::Boolean => fuzz_bool(rng),
                    form::TypeValue::Int8 => fuzz_i8(rng),
                    form::TypeValue::Uint8 => fuzz_u8(rng),
                    form::TypeValue::Int16 => fuzz_i16(rng),
                    form::TypeValue::Uint16 => fuzz_u16(rng),
                    form::TypeValue::Int32 => fuzz_i32(rng),
                    form::TypeValue::Uint32 => fuzz_u32(rng),
                    form::TypeValue::Float32 => fuzz_f32(rng),
                    form::TypeValue::Float64 => fuzz_f64(rng),
                    form::TypeValue::String => fuzz_string(rng),
                    form::TypeValue::Timestamp => fuzz_timestamp(rng),
                }
            }
        }
        Form::Enum(form::Enum {
            ref values,
            nullable,
        }) => {
            if nullable && rng.gen() {
                Value::Null
            } else {
                fuzz_enum(rng, values)
            }
        }
        Form::Elements(form::Elements {
            ref schema,
            nullable,
        }) => {
            if nullable && rng.gen() {
                Value::Null
            } else {
                fuzz_elems(root, rng, schema)
            }
        }
        Form::Properties(form::Properties {
            ref required,
            ref optional,
            additional,
            nullable,
            ..
        }) => {
            if nullable && rng.gen() {
                Value::Null
            } else {
                fuzz_props(root, rng, required, optional, additional)
            }
        }
        Form::Values(form::Values {
            ref schema,
            nullable,
        }) => {
            if nullable && rng.gen() {
                Value::Null
            } else {
                fuzz_values(root, rng, schema)
            }
        }
        Form::Discriminator(form::Discriminator {
            ref mapping,
            ref discriminator,
            nullable,
        }) => {
            if nullable && rng.gen() {
                Value::Null
            } else {
                fuzz_discr(root, rng, discriminator, mapping)
            }
        }
    }
}

fn fuzz_any<R: rand::Rng + ?Sized>(root: &Schema, rng: &mut R) -> Value {
    match rng.gen_range(0, 7) {
        0 => Value::Null,
        1 => fuzz_bool(rng),
        2 => fuzz_u8(rng),
        3 => fuzz_f64(rng),
        4 => fuzz_string(rng),
        5 => fuzz_elems(root, rng, &Default::default()),
        6 => fuzz_values(root, rng, &Default::default()),
        _ => unreachable!(),
    }
}

fn fuzz_bool<R: rand::Rng + ?Sized>(rng: &mut R) -> Value {
    rng.gen::<bool>().into()
}

fn fuzz_i8<R: rand::Rng + ?Sized>(rng: &mut R) -> Value {
    rng.gen::<i8>().into()
}

fn fuzz_u8<R: rand::Rng + ?Sized>(rng: &mut R) -> Value {
    rng.gen::<u8>().into()
}

fn fuzz_i16<R: rand::Rng + ?Sized>(rng: &mut R) -> Value {
    rng.gen::<i16>().into()
}

fn fuzz_u16<R: rand::Rng + ?Sized>(rng: &mut R) -> Value {
    rng.gen::<u16>().into()
}

fn fuzz_i32<R: rand::Rng + ?Sized>(rng: &mut R) -> Value {
    rng.gen::<i32>().into()
}

fn fuzz_u32<R: rand::Rng + ?Sized>(rng: &mut R) -> Value {
    rng.gen::<u32>().into()
}

fn fuzz_f32<R: rand::Rng + ?Sized>(rng: &mut R) -> Value {
    rng.gen::<f32>().into()
}

fn fuzz_f64<R: rand::Rng + ?Sized>(rng: &mut R) -> Value {
    rng.gen::<f64>().into()
}

fn fuzz_str<R: rand::Rng + ?Sized>(rng: &mut R) -> String {
    (0..rng.gen_range(0, 8))
        .map(|_| rng.gen_range(32u8, 127u8) as char)
        .collect::<String>()
}

fn fuzz_string<R: rand::Rng + ?Sized>(rng: &mut R) -> Value {
    fuzz_str(rng).into()
}

fn fuzz_timestamp<R: rand::Rng + ?Sized>(rng: &mut R) -> Value {
    let date_time = NaiveDateTime::from_timestamp(rng.gen::<i32>() as i64, 0);
    let date_time = DateTime::<Utc>::from_utc(date_time, Utc);
    date_time.to_rfc3339().into()
}

fn fuzz_enum<R: rand::Rng + ?Sized>(rng: &mut R, vals: &HashSet<String>) -> Value {
    vals.iter().choose(rng).unwrap().clone().into()
}

fn fuzz_elems<R: rand::Rng + ?Sized>(root: &Schema, rng: &mut R, sub_schema: &Schema) -> Value {
    (0..rng.gen_range(0, 8))
        .map(|_| fuzz(root, rng, sub_schema))
        .collect::<Vec<_>>()
        .into()
}

fn fuzz_props<R: rand::Rng + ?Sized>(
    root: &Schema,
    rng: &mut R,
    required: &HashMap<String, Schema>,
    optional: &HashMap<String, Schema>,
    additional: bool,
) -> Value {
    let mut vals = Vec::new();

    for (k, v) in required {
        vals.push((k.clone(), fuzz(root, rng, v)));
    }

    for (k, v) in optional {
        if rng.gen() {
            vals.push((k.clone(), fuzz(root, rng, v)));
        }
    }

    if additional {
        for _ in 0..rng.gen_range(0, 8) {
            vals.push((fuzz_str(rng), fuzz_any(root, rng)));
        }
    }

    vals.into_iter()
        .collect::<serde_json::Map<String, Value>>()
        .into()
}

fn fuzz_values<R: rand::Rng + ?Sized>(root: &Schema, rng: &mut R, sub_schema: &Schema) -> Value {
    (0..rng.gen_range(0, 8))
        .map(|_| {
            (
                fuzz_string(rng).as_str().unwrap().to_owned(),
                fuzz(root, rng, sub_schema),
            )
        })
        .collect::<serde_json::Map<String, Value>>()
        .into()
}

fn fuzz_discr<R: rand::Rng + ?Sized>(
    root: &Schema,
    rng: &mut R,
    tag: &str,
    mapping: &HashMap<String, Schema>,
) -> Value {
    let (tag_val, sub_schema) = mapping.iter().choose(rng).unwrap();
    let mut obj = fuzz(root, rng, sub_schema);
    obj.as_object_mut()
        .unwrap()
        .insert(tag.to_owned(), tag_val.clone().into());
    obj
}
