//! Generate fuzzed data from a JSON Type Definition schema.
//!
//! # Quick start
//!
//! Here's how you can use [`fuzz`] to generate dummy data from a schema.
//!
//! ```
//! use serde_json::json;
//! use rand::SeedableRng;
//!
//! // An example schema we can test against.
//! let schema = jtd::Schema::from_serde_schema(serde_json::from_value(json!({
//!     "properties": {
//!         "name": { "type": "string" },
//!         "createdAt": { "type": "timestamp" },
//!         "favoriteNumbers": {
//!             "elements": { "type": "uint8" }
//!         }
//!     }
//! })).unwrap()).unwrap();
//!
//! // A hard-coded RNG, so that the output is predictable.
//! let mut rng = rand_pcg::Pcg32::seed_from_u64(8927);
//!
//! assert_eq!(jtd_fuzz::fuzz(&schema, &mut rng), json!({
//!     "name": "e",
//!     "createdAt": "1931-10-18T16:37:09-03:03",
//!     "favoriteNumbers": [166, 142]
//! }));
//! ```

use jtd::{Schema, Type};
use rand::seq::IteratorRandom;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

// Max length when generating "sequences" of things, such as strings, arrays,
// and objects.
const MAX_SEQ_LENGTH: u8 = 8;

// Key in metadata that, if present and one of the recognized values, will
// result in a specific sort of data being produced instead of the generic
// default.
const METADATA_KEY_FUZZ_HINT: &'static str = "fuzzHint";

/// Generates a single random JSON value satisfying a given schema.
///
/// The generated output is purely a function of the given schema and RNG. It is
/// guaranteed that the returned data satisfies the given schema.
///
/// # Invariants for generated data
///
/// The output of this function is not guaranteed to remain the same between
/// different versions of this crate; if you use a different version of this
/// crate, you may get different output from this function.
///
/// Some properties of fuzz which are guaranteed for this version of the crate,
/// but which may change within the same major version number of the crate:
///
/// * Generated strings (for `type: string` and object keys), arrays (for
///   `elements`), and objects (for `values`) will have no more than seven
///   characters, elements, and members, respectively.
///
/// * No more than seven "extra" properties will be added for schemas with
///   `additionalProperties`.
///
/// * Generated strings will be entirely printable ASCII.
///
/// * Generated timestamps will have a random offset from UTC. These offsets
///   will not necessarily be "historical"; some offsets may never have been
///   used in the real world.
///
/// # Using `fuzzHint`
///
/// If you want to generate a specific sort of string from your schema, you can
/// use the `fuzzHint` metadata property to customize output. For example, if
/// you'd like to generate a fake email instead of a generic string, you can use
/// a `fuzzHint` of `en_us/internet/email`:
///
/// ```
/// use serde_json::json;
/// use rand::SeedableRng;
///
/// let schema = jtd::Schema::from_serde_schema(serde_json::from_value(json!({
///     "type": "string",
///     "metadata": {
///         "fuzzHint": "en_us/internet/email"
///     }
/// })).unwrap()).unwrap();
///
/// let mut rng = rand_pcg::Pcg32::seed_from_u64(8927);
/// assert_eq!(jtd_fuzz::fuzz(&schema, &mut rng), json!("prenner3@fay.com"));
/// ```
///
/// `fuzzHint` will only be honored for schemas with `type` of `string`. It will
/// not be honored for empty schemas. If `fuzzHint` does not have one of the
/// values listed below, then its value will be ignored.
///
/// The possible values for `fuzzHint` are:
///
/// * [`en_us/addresses/city_name`][`faker_rand::en_us::addresses::CityName`]
/// * [`en_us/addresses/division_abbreviation`][`faker_rand::en_us::addresses::DivisionAbbreviation`]
/// * [`en_us/addresses/division`][`faker_rand::en_us::addresses::Division`]
/// * [`en_us/addresses/postal_code`][`faker_rand::en_us::addresses::PostalCode`]
/// * [`en_us/addresses/secondary_address`][`faker_rand::en_us::addresses::SecondaryAddress`]
/// * [`en_us/addresses/street_address`][`faker_rand::en_us::addresses::StreetAddress`]
/// * [`en_us/addresses/street_name`][`faker_rand::en_us::addresses::StreetName`]
/// * [`en_us/company/company_name`][`faker_rand::en_us::company::CompanyName`]
/// * [`en_us/company/slogan`][`faker_rand::en_us::company::Slogan`]
/// * [`en_us/internet/domain`][`faker_rand::en_us::internet::Domain`]
/// * [`en_us/internet/email`][`faker_rand::en_us::internet::Email`]
/// * [`en_us/internet/username`][`faker_rand::en_us::internet::Username`]
/// * [`en_us/names/first_name`][`faker_rand::en_us::names::FirstName`]
/// * [`en_us/names/full_name`][`faker_rand::en_us::names::FullName`]
/// * [`en_us/names/last_name`][`faker_rand::en_us::names::LastName`]
/// * [`en_us/names/name_prefix`][`faker_rand::en_us::names::NamePrefix`]
/// * [`en_us/names/name_suffix`][`faker_rand::en_us::names::NameSuffix`]
/// * [`en_us/phones/phone_number`][`faker_rand::en_us::phones::PhoneNumber`]
/// * [`fr_fr/addresses/address`][`faker_rand::fr_fr::addresses::Address`]
/// * [`fr_fr/addresses/city_name`][`faker_rand::fr_fr::addresses::CityName`]
/// * [`fr_fr/addresses/division`][`faker_rand::fr_fr::addresses::Division`]
/// * [`fr_fr/addresses/postal_code`][`faker_rand::fr_fr::addresses::PostalCode`]
/// * [`fr_fr/addresses/secondary_address`][`faker_rand::fr_fr::addresses::SecondaryAddress`]
/// * [`fr_fr/addresses/street_address`][`faker_rand::fr_fr::addresses::StreetAddress`]
/// * [`fr_fr/addresses/street_name`][`faker_rand::fr_fr::addresses::StreetName`]
/// * [`fr_fr/company/company_name`][`faker_rand::fr_fr::company::CompanyName`]
/// * [`fr_fr/internet/domain`][`faker_rand::fr_fr::internet::Domain`]
/// * [`fr_fr/internet/email`][`faker_rand::fr_fr::internet::Email`]
/// * [`fr_fr/internet/username`][`faker_rand::fr_fr::internet::Username`]
/// * [`fr_fr/names/first_name`][`faker_rand::fr_fr::names::FirstName`]
/// * [`fr_fr/names/full_name`][`faker_rand::fr_fr::names::FullName`]
/// * [`fr_fr/names/last_name`][`faker_rand::fr_fr::names::LastName`]
/// * [`fr_fr/names/name_prefix`][`faker_rand::fr_fr::names::NamePrefix`]
/// * [`fr_fr/phones/phone_number`][`faker_rand::fr_fr::phones::PhoneNumber`]
/// * [`lorem/word`][`faker_rand::lorem::Word`]
/// * [`lorem/sentence`][`faker_rand::lorem::Sentence`]
/// * [`lorem/paragraph`][`faker_rand::lorem::Paragraph`]
/// * [`lorem/paragraphs`][`faker_rand::lorem::Paragraphs`]
///
/// New acceptable values for `fuzzHint` may be added to this crate within the
/// same major version.
pub fn fuzz<R: rand::Rng>(schema: &Schema, rng: &mut R) -> Value {
    fuzz_with_root(schema, rng, schema)
}

fn fuzz_with_root<R: rand::Rng>(root: &Schema, rng: &mut R, schema: &Schema) -> Value {
    match schema {
        Schema::Empty { .. } => {
            // Generate one of null, boolean, uint8, float64, string, the
            // elements form, or the values form. The reasoning is that it's
            // reasonable behavior, and has a good chance of helping users catch
            // bugs.
            //
            // As a bit of a hack, we here try to detect if we are the fuzzing
            // root schema. If we are, we will allow ourselves to generate
            // structures which themselves will recursively contain more empty
            // schemas. But those empty schemas in turn will not contain further
            // empty schemas.
            //
            // Doing so helps us avoid overflowing the stack.
            let range_max_value = if root as *const _ == schema as *const _ {
                7 // 0 through 6
            } else {
                5 // 0 through 4
            };

            let val = rng.gen_range(0..range_max_value);
            match val {
                // 0-4 are cases we will always potentially generate.
                0 => Value::Null,
                1 => rng.gen::<bool>().into(),
                2 => rng.gen::<u8>().into(),
                3 => rng.gen::<f64>().into(),
                4 => fuzz_string(rng).into(),

                // All the following cases are "recursive" cases. See above for
                // why it's important these come after the "primitive" cases.
                5 => {
                    let schema = Schema::Elements {
                        metadata: Default::default(),
                        definitions: Default::default(),
                        nullable: false,
                        elements: Box::new(Schema::Empty {
                            metadata: Default::default(),
                            definitions: Default::default(),
                        }),
                    };

                    fuzz(&schema, rng)
                }

                6 => {
                    let schema = Schema::Values {
                        metadata: Default::default(),
                        definitions: Default::default(),
                        nullable: false,
                        values: Box::new(Schema::Empty {
                            metadata: Default::default(),
                            definitions: Default::default(),
                        }),
                    };

                    fuzz(&schema, rng)
                }

                _ => unreachable!(),
            }
        }

        Schema::Ref {
            ref ref_, nullable, ..
        } => {
            if *nullable && rng.gen() {
                return Value::Null;
            }

            fuzz_with_root(root, rng, &root.definitions()[ref_])
        }

        Schema::Type {
            ref metadata,
            ref type_,
            nullable,
            ..
        } => {
            if *nullable && rng.gen() {
                return Value::Null;
            }

            match type_ {
                Type::Boolean => rng.gen::<bool>().into(),
                Type::Float32 => rng.gen::<f32>().into(),
                Type::Float64 => rng.gen::<f64>().into(),
                Type::Int8 => rng.gen::<i8>().into(),
                Type::Uint8 => rng.gen::<u8>().into(),
                Type::Int16 => rng.gen::<i16>().into(),
                Type::Uint16 => rng.gen::<u16>().into(),
                Type::Int32 => rng.gen::<i32>().into(),
                Type::Uint32 => rng.gen::<u32>().into(),
                Type::String => {
                    match metadata.get(METADATA_KEY_FUZZ_HINT).and_then(Value::as_str) {
                        Some("en_us/addresses/address") => rng
                            .gen::<faker_rand::en_us::addresses::Address>()
                            .to_string()
                            .into(),
                        Some("en_us/addresses/city_name") => rng
                            .gen::<faker_rand::en_us::addresses::CityName>()
                            .to_string()
                            .into(),
                        Some("en_us/addresses/division") => rng
                            .gen::<faker_rand::en_us::addresses::Division>()
                            .to_string()
                            .into(),
                        Some("en_us/addresses/division_abbreviation") => rng
                            .gen::<faker_rand::en_us::addresses::DivisionAbbreviation>()
                            .to_string()
                            .into(),
                        Some("en_us/addresses/postal_code") => rng
                            .gen::<faker_rand::en_us::addresses::PostalCode>()
                            .to_string()
                            .into(),
                        Some("en_us/addresses/secondary_address") => rng
                            .gen::<faker_rand::en_us::addresses::SecondaryAddress>()
                            .to_string()
                            .into(),
                        Some("en_us/addresses/street_address") => rng
                            .gen::<faker_rand::en_us::addresses::StreetAddress>()
                            .to_string()
                            .into(),
                        Some("en_us/addresses/street_name") => rng
                            .gen::<faker_rand::en_us::addresses::StreetName>()
                            .to_string()
                            .into(),
                        Some("en_us/company/company_name") => rng
                            .gen::<faker_rand::en_us::company::CompanyName>()
                            .to_string()
                            .into(),
                        Some("en_us/company/slogan") => rng
                            .gen::<faker_rand::en_us::company::Slogan>()
                            .to_string()
                            .into(),
                        Some("en_us/internet/domain") => rng
                            .gen::<faker_rand::en_us::internet::Domain>()
                            .to_string()
                            .into(),
                        Some("en_us/internet/email") => rng
                            .gen::<faker_rand::en_us::internet::Email>()
                            .to_string()
                            .into(),
                        Some("en_us/internet/username") => rng
                            .gen::<faker_rand::en_us::internet::Username>()
                            .to_string()
                            .into(),
                        Some("en_us/names/first_name") => rng
                            .gen::<faker_rand::en_us::names::FirstName>()
                            .to_string()
                            .into(),
                        Some("en_us/names/full_name") => rng
                            .gen::<faker_rand::en_us::names::FullName>()
                            .to_string()
                            .into(),
                        Some("en_us/names/last_name") => rng
                            .gen::<faker_rand::en_us::names::LastName>()
                            .to_string()
                            .into(),
                        Some("en_us/names/name_prefix") => rng
                            .gen::<faker_rand::en_us::names::NamePrefix>()
                            .to_string()
                            .into(),
                        Some("en_us/names/name_suffix") => rng
                            .gen::<faker_rand::en_us::names::NameSuffix>()
                            .to_string()
                            .into(),
                        Some("en_us/phones/phone_number") => rng
                            .gen::<faker_rand::en_us::phones::PhoneNumber>()
                            .to_string()
                            .into(),
                        Some("fr_fr/addresses/address") => rng
                            .gen::<faker_rand::fr_fr::addresses::Address>()
                            .to_string()
                            .into(),
                        Some("fr_fr/addresses/city_name") => rng
                            .gen::<faker_rand::fr_fr::addresses::CityName>()
                            .to_string()
                            .into(),
                        Some("fr_fr/addresses/division") => rng
                            .gen::<faker_rand::fr_fr::addresses::Division>()
                            .to_string()
                            .into(),
                        Some("fr_fr/addresses/postal_code") => rng
                            .gen::<faker_rand::fr_fr::addresses::PostalCode>()
                            .to_string()
                            .into(),
                        Some("fr_fr/addresses/secondary_address") => rng
                            .gen::<faker_rand::fr_fr::addresses::SecondaryAddress>()
                            .to_string()
                            .into(),
                        Some("fr_fr/addresses/street_address") => rng
                            .gen::<faker_rand::fr_fr::addresses::StreetAddress>()
                            .to_string()
                            .into(),
                        Some("fr_fr/addresses/street_name") => rng
                            .gen::<faker_rand::fr_fr::addresses::StreetName>()
                            .to_string()
                            .into(),
                        Some("fr_fr/company/company_name") => rng
                            .gen::<faker_rand::fr_fr::company::CompanyName>()
                            .to_string()
                            .into(),
                        Some("fr_fr/internet/domain") => rng
                            .gen::<faker_rand::fr_fr::internet::Domain>()
                            .to_string()
                            .into(),
                        Some("fr_fr/internet/email") => rng
                            .gen::<faker_rand::fr_fr::internet::Email>()
                            .to_string()
                            .into(),
                        Some("fr_fr/internet/username") => rng
                            .gen::<faker_rand::fr_fr::internet::Username>()
                            .to_string()
                            .into(),
                        Some("fr_fr/names/first_name") => rng
                            .gen::<faker_rand::fr_fr::names::FirstName>()
                            .to_string()
                            .into(),
                        Some("fr_fr/names/full_name") => rng
                            .gen::<faker_rand::fr_fr::names::FullName>()
                            .to_string()
                            .into(),
                        Some("fr_fr/names/last_name") => rng
                            .gen::<faker_rand::fr_fr::names::LastName>()
                            .to_string()
                            .into(),
                        Some("fr_fr/names/name_prefix") => rng
                            .gen::<faker_rand::fr_fr::names::NamePrefix>()
                            .to_string()
                            .into(),
                        Some("fr_fr/phones/phone_number") => rng
                            .gen::<faker_rand::fr_fr::phones::PhoneNumber>()
                            .to_string()
                            .into(),
                        Some("lorem/word") => {
                            rng.gen::<faker_rand::lorem::Word>().to_string().into()
                        }
                        Some("lorem/sentence") => {
                            rng.gen::<faker_rand::lorem::Sentence>().to_string().into()
                        }
                        Some("lorem/paragraph") => {
                            rng.gen::<faker_rand::lorem::Paragraph>().to_string().into()
                        }
                        Some("lorem/paragraphs") => rng
                            .gen::<faker_rand::lorem::Paragraphs>()
                            .to_string()
                            .into(),

                        _ => fuzz_string(rng).into(),
                    }
                }
                Type::Timestamp => {
                    use chrono::TimeZone;

                    // We'll generate timestamps with some random seconds offset
                    // from UTC. Most of these random offsets will never have
                    // been used historically, but they can nonetheless be used
                    // in valid RFC3339 timestamps.
                    //
                    // Although timestamp_millis accepts an i64, not all values
                    // in that range are permissible. The i32 range is entirely
                    // safe.
                    //
                    // However, UTC offsets present a practical complication:
                    //
                    // Java's java.time.ZoneOffset restricts offsets to no more
                    // than 18 hours from UTC:
                    //
                    // https://docs.oracle.com/javase/8/docs/api/java/time/ZoneOffset.html
                    //
                    // .NET's System.DateTimeOffset restricts offsets to no more
                    // than 14 hours from UTC:
                    //
                    // https://docs.microsoft.com/en-us/dotnet/api/system.datetimeoffset.tooffset?view=net-5.0
                    //
                    // To make jtd-fuzz work out of the box with these
                    // ecosystems, we will limit ourselves to the most selective
                    // of these time ranges.
                    let max_offset = 14 * 60 * 60;
                    chrono::FixedOffset::east(rng.gen_range(-max_offset..=max_offset))
                        .timestamp(rng.gen::<i32>() as i64, 0)
                        .to_rfc3339()
                        .into()
                }
            }
        }

        Schema::Enum {
            ref enum_,
            nullable,
            ..
        } => {
            if *nullable && rng.gen() {
                return Value::Null;
            }

            enum_.iter().choose(rng).unwrap().clone().into()
        }

        Schema::Elements {
            ref elements,
            nullable,
            ..
        } => {
            if *nullable && rng.gen() {
                return Value::Null;
            }

            (0..rng.gen_range(0..=MAX_SEQ_LENGTH))
                .map(|_| fuzz_with_root(root, rng, elements))
                .collect::<Vec<_>>()
                .into()
        }

        Schema::Properties {
            ref properties,
            ref optional_properties,
            additional_properties,
            nullable,
            ..
        } => {
            if *nullable && rng.gen() {
                return Value::Null;
            }

            let mut members = BTreeMap::new();

            let mut required_keys: Vec<_> = properties.keys().cloned().collect();
            required_keys.sort();

            for k in required_keys {
                let v = fuzz_with_root(root, rng, &properties[&k]);
                members.insert(k, v);
            }

            let mut optional_keys: Vec<_> = optional_properties.keys().cloned().collect();
            optional_keys.sort();

            for k in optional_keys {
                if rng.gen() {
                    continue;
                }

                let v = fuzz_with_root(root, rng, &optional_properties[&k]);
                members.insert(k, v);
            }

            if *additional_properties {
                // Go's encoding/json package, which implements JSON
                // serialization/deserialization, is case-insensitive on inputs.
                //
                // In order to generate fuzzed data that's compatible with Go,
                // we'll avoid generating "additional" properties that are
                // case-insensitively equal to any required or optional property
                // from the schema.
                //
                // Since we'll only generate ASCII properties here, we don't
                // need to worry about implementing proper Unicode folding.
                let defined_properties_lowercase: BTreeSet<_> = properties
                    .keys()
                    .chain(optional_properties.keys())
                    .map(|s| s.to_lowercase())
                    .collect();

                for _ in 0..rng.gen_range(0..=MAX_SEQ_LENGTH) {
                    let key = fuzz_string(rng);

                    if !defined_properties_lowercase.contains(&key.to_lowercase()) {
                        members.insert(
                            key,
                            fuzz(
                                &Schema::Empty {
                                    metadata: Default::default(),
                                    definitions: Default::default(),
                                },
                                rng,
                            ),
                        );
                    }
                }
            }

            members
                .into_iter()
                .collect::<serde_json::Map<String, Value>>()
                .into()
        }

        Schema::Values {
            ref values,
            nullable,
            ..
        } => {
            if *nullable && rng.gen() {
                return Value::Null;
            }

            (0..rng.gen_range(0..=MAX_SEQ_LENGTH))
                .map(|_| (fuzz_string(rng), fuzz_with_root(root, rng, values)))
                .collect::<serde_json::Map<String, Value>>()
                .into()
        }

        Schema::Discriminator {
            ref mapping,
            ref discriminator,
            nullable,
            ..
        } => {
            if *nullable && rng.gen() {
                return Value::Null;
            }

            let (discriminator_value, sub_schema) = mapping.iter().choose(rng).unwrap();

            let mut obj = fuzz_with_root(root, rng, sub_schema);
            obj.as_object_mut().unwrap().insert(
                discriminator.to_owned(),
                discriminator_value.to_owned().into(),
            );
            obj
        }
    }
}

fn fuzz_string<R: rand::Rng>(rng: &mut R) -> String {
    (0..rng.gen_range(0..=MAX_SEQ_LENGTH))
        .map(|_| rng.gen_range(32u8..=127u8) as char)
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_fuzz_empty() {
        assert_valid_fuzz(json!({}));
    }

    #[test]
    fn test_fuzz_ref() {
        assert_valid_fuzz(json!({
            "definitions": {
                "a": { "type": "timestamp" },
                "b": { "type": "timestamp", "nullable": true },
                "c": { "ref": "b" },
            },
            "properties": {
                "a": { "ref": "a" },
                "b": { "ref": "b" },
                "c": { "ref": "c" },
            }
        }));
    }

    #[test]
    fn test_fuzz_type() {
        assert_valid_fuzz(json!({ "type": "boolean" }));
        assert_valid_fuzz(json!({ "type": "boolean", "nullable": true }));
        assert_valid_fuzz(json!({ "type": "float32" }));
        assert_valid_fuzz(json!({ "type": "float32", "nullable": true }));
        assert_valid_fuzz(json!({ "type": "float64" }));
        assert_valid_fuzz(json!({ "type": "float64", "nullable": true }));
        assert_valid_fuzz(json!({ "type": "int8" }));
        assert_valid_fuzz(json!({ "type": "int8", "nullable": true }));
        assert_valid_fuzz(json!({ "type": "uint8" }));
        assert_valid_fuzz(json!({ "type": "uint8", "nullable": true }));
        assert_valid_fuzz(json!({ "type": "uint16" }));
        assert_valid_fuzz(json!({ "type": "uint16", "nullable": true }));
        assert_valid_fuzz(json!({ "type": "uint32" }));
        assert_valid_fuzz(json!({ "type": "uint32", "nullable": true }));
        assert_valid_fuzz(json!({ "type": "string" }));
        assert_valid_fuzz(json!({ "type": "string", "nullable": true }));
        assert_valid_fuzz(json!({ "type": "timestamp" }));
        assert_valid_fuzz(json!({ "type": "timestamp", "nullable": true }));
    }

    #[test]
    fn test_fuzz_enum() {
        assert_valid_fuzz(json!({ "enum": ["a", "b", "c" ]}));
        assert_valid_fuzz(json!({ "enum": ["a", "b", "c" ], "nullable": true }));
    }

    #[test]
    fn test_fuzz_elements() {
        assert_valid_fuzz(json!({ "elements": { "type": "uint8" }}));
        assert_valid_fuzz(json!({ "elements": { "type": "uint8" }, "nullable": true }));
    }

    #[test]
    fn test_fuzz_properties() {
        assert_valid_fuzz(json!({
            "properties": {
                "a": { "type": "uint8" },
                "b": { "type": "string" },
            },
            "optionalProperties": {
                "c": { "type": "uint32" },
                "d": { "type": "timestamp" },
            },
            "additionalProperties": true,
            "nullable": true,
        }));
    }

    #[test]
    fn test_fuzz_values() {
        assert_valid_fuzz(json!({ "values": { "type": "uint8" }}));
        assert_valid_fuzz(json!({ "values": { "type": "uint8" }, "nullable": true }));
    }

    #[test]
    fn test_fuzz_discriminator() {
        assert_valid_fuzz(json!({
            "discriminator": "version",
            "mapping": {
                "v1": {
                    "properties": {
                        "foo": { "type": "string" },
                        "bar": { "type": "timestamp" }
                    }
                },
                "v2": {
                    "properties": {
                        "foo": { "type": "uint8" },
                        "bar": { "type": "float32" }
                    }
                }
            },
            "nullable": true,
        }));
    }

    fn assert_valid_fuzz(schema: Value) {
        use rand::SeedableRng;

        let mut rng = rand_pcg::Pcg32::seed_from_u64(8927);
        let schema = Schema::from_serde_schema(serde_json::from_value(schema).unwrap()).unwrap();

        // Poor man's fuzzing.
        for _ in 0..1000 {
            let instance = super::fuzz(&schema, &mut rng);
            let errors = jtd::validate(&schema, &instance, Default::default()).unwrap();
            assert!(errors.is_empty(), "{}", instance);
        }
    }
}
