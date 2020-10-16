//! Generate fuzzed data from a JSON Type Definition schema.

use rand::seq::IteratorRandom;
use serde_json::Value;
use std::collections::BTreeMap;

// Max length when generating "sequences" of things, such as strings, arrays,
// and objects.
const MAX_SEQ_LENGTH: u8 = 8;

/// Generates a single random JSON value satisfying a given schema.
///
/// The generated output is purely a function of the given schema and RNG. It is
/// guaranteed that the returned data satisfies the given schema.
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
/// As an example of the sort of data this function may produce:
///
/// ```
/// use std::convert::TryInto;
/// use serde_json::json;
/// use rand::SeedableRng;
///
/// // An example schema we can test against.
/// let schema: jtd::SerdeSchema = serde_json::from_value(json!({
///     "properties": {
///         "name": { "type": "string" },
///         "createdAt": { "type": "timestamp" },
///         "favoriteNumbers": {
///             "elements": { "type": "uint8" }
///         }
///     }
/// })).unwrap();
///
/// let schema: jtd::Schema = schema.try_into().unwrap();
///
/// // A hard-coded RNG, so that the output is predictable.
/// let mut rng = rand_pcg::Pcg32::seed_from_u64(8927);
///
/// assert_eq!(jtd_fuzz::fuzz(&schema, &mut rng), json!({
///     "name": "e",
///     "createdAt": "1931-10-18T14:26:10-05:14",
///     "favoriteNumbers": [166, 142]
/// }));
/// ```
pub fn fuzz<R: rand::Rng>(schema: &jtd::Schema, rng: &mut R) -> Value {
    fuzz_with_root(schema, rng, schema)
}

fn fuzz_with_root<R: rand::Rng>(root: &jtd::Schema, rng: &mut R, schema: &jtd::Schema) -> Value {
    match schema.form {
        jtd::Form::Empty => {
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

            let val = rng.gen_range(0, range_max_value);
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
                    let schema = jtd::Schema {
                        metadata: BTreeMap::new(),
                        definitions: BTreeMap::new(),
                        form: jtd::Form::Elements(jtd::form::Elements {
                            nullable: false,
                            schema: Default::default(),
                        }),
                    };

                    fuzz(&schema, rng)
                }

                6 => {
                    let schema = jtd::Schema {
                        metadata: BTreeMap::new(),
                        definitions: BTreeMap::new(),
                        form: jtd::Form::Values(jtd::form::Values {
                            nullable: false,
                            schema: Default::default(),
                        }),
                    };

                    fuzz(&schema, rng)
                }

                _ => unreachable!(),
            }
        }

        jtd::Form::Ref(jtd::form::Ref {
            ref definition,
            nullable,
        }) => {
            if nullable && rng.gen() {
                return Value::Null;
            }

            fuzz_with_root(root, rng, &root.definitions[definition])
        }

        jtd::Form::Type(jtd::form::Type {
            ref type_value,
            nullable,
        }) => {
            if nullable && rng.gen() {
                return Value::Null;
            }

            match type_value {
                jtd::form::TypeValue::Boolean => rng.gen::<bool>().into(),
                jtd::form::TypeValue::Float32 => rng.gen::<f32>().into(),
                jtd::form::TypeValue::Float64 => rng.gen::<f64>().into(),
                jtd::form::TypeValue::Int8 => rng.gen::<i8>().into(),
                jtd::form::TypeValue::Uint8 => rng.gen::<u8>().into(),
                jtd::form::TypeValue::Int16 => rng.gen::<i16>().into(),
                jtd::form::TypeValue::Uint16 => rng.gen::<u16>().into(),
                jtd::form::TypeValue::Int32 => rng.gen::<i32>().into(),
                jtd::form::TypeValue::Uint32 => rng.gen::<u32>().into(),
                jtd::form::TypeValue::String => fuzz_string(rng).into(),
                jtd::form::TypeValue::Timestamp => {
                    use chrono::TimeZone;

                    // For timestamp generation, we're going to be real
                    // psychotic.
                    //
                    // We'll generate timestamps with some random seconds offset
                    // from UTC. Most of these random offsets will never have
                    // been used historically, but they can nonetheless be used
                    // in valid RFC3339 timestamps.
                    //
                    // Although timestamp_millis accepts an i64, not all values
                    // in that range are permissible. The i32 range is entirely
                    // safe.
                    chrono::FixedOffset::east(rng.gen_range(-86_400 + 1, 86_400 - 1))
                        .timestamp(rng.gen::<i32>() as i64, 0)
                        .to_rfc3339()
                        .into()
                }
            }
        }

        jtd::Form::Enum(jtd::form::Enum {
            ref values,
            nullable,
        }) => {
            if nullable && rng.gen() {
                return Value::Null;
            }

            values.iter().choose(rng).unwrap().clone().into()
        }

        jtd::Form::Elements(jtd::form::Elements {
            schema: ref sub_schema,
            nullable,
        }) => {
            if nullable && rng.gen() {
                return Value::Null;
            }

            (0..rng.gen_range(0, MAX_SEQ_LENGTH))
                .map(|_| fuzz_with_root(root, rng, sub_schema))
                .collect::<Vec<_>>()
                .into()
        }

        jtd::Form::Properties(jtd::form::Properties {
            ref required,
            ref optional,
            additional,
            nullable,
            ..
        }) => {
            if nullable && rng.gen() {
                return Value::Null;
            }

            let mut members = BTreeMap::new();

            let mut required_keys: Vec<_> = required.keys().cloned().collect();
            required_keys.sort();

            for k in required_keys {
                let v = fuzz_with_root(root, rng, &required[&k]);
                members.insert(k, v);
            }

            let mut optional_keys: Vec<_> = optional.keys().cloned().collect();
            optional_keys.sort();

            for k in optional_keys {
                if rng.gen() {
                    continue;
                }

                let v = fuzz_with_root(root, rng, &optional[&k]);
                members.insert(k, v);
            }

            if additional {
                for _ in 0..rng.gen_range(0, MAX_SEQ_LENGTH) {
                    let key = fuzz_string(rng);

                    // It would be wrong for this code to check if
                    // members.contains_key, because that would leave open the
                    // possibility for this code to produce an optional property
                    // that we elected not to generate previously.
                    if !required.contains_key(&key) && !optional.contains_key(&key) {
                        members.insert(key, fuzz_with_root(root, rng, &Default::default()));
                    }
                }
            }

            members
                .into_iter()
                .collect::<serde_json::Map<String, Value>>()
                .into()
        }

        jtd::Form::Values(jtd::form::Values {
            schema: ref sub_schema,
            nullable,
        }) => {
            if nullable && rng.gen() {
                return Value::Null;
            }

            (0..rng.gen_range(0, MAX_SEQ_LENGTH))
                .map(|_| (fuzz_string(rng), fuzz_with_root(root, rng, sub_schema)))
                .collect::<serde_json::Map<String, Value>>()
                .into()
        }

        jtd::Form::Discriminator(jtd::form::Discriminator {
            ref mapping,
            ref discriminator,
            nullable,
        }) => {
            if nullable && rng.gen() {
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
    (0..rng.gen_range(0, MAX_SEQ_LENGTH))
        .map(|_| rng.gen_range(32u8, 127u8) as char)
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};

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
        use std::convert::TryInto;

        let schema: jtd::SerdeSchema = serde_json::from_value(schema).unwrap();
        let schema: jtd::Schema = schema.try_into().unwrap();
        let mut rng = rand_pcg::Pcg32::seed_from_u64(8927);

        let validator = jtd::Validator {
            max_errors: None,
            max_depth: None,
        };

        // Poor man's fuzzing.
        for _ in 0..1000 {
            let instance = super::fuzz(&schema, &mut rng);
            let errors = validator.validate(&schema, &instance).unwrap();
            assert!(errors.is_empty(), "{}", instance);
        }
    }
}
