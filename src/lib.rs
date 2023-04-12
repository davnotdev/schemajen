//! # SchemaJen
//!
//!
//! Auto-magically infer language bindings given a JSON schema.
//!
//! If you are a user and not a contributor, go back to the [Github page](https://github.com/davnotdev/schemajen) as this is *probably* not for you.
//!
//! ## Usage 
//!
//! ```
//! use schemajen::*;
//!
//! //  See [`ACCUMULATOR_SUPPORT_LIST`] for string options.
//! //  let mut accumulator = accumulator_choose_with_str("rust");
//!
//! let mut accumulator = Box::new(RustAccumulator::begin());
//! let res = generate(&mut accumulator, "MyType", r#"{"a": 10}"#);
//! res.unwrap();
//! eprintln!("{}", res);
//!
//! ```
//!
//! ## Contributing
//!
//! All code generators (aka accumulators) are found in [`codegen`].
//!

pub mod codegen;

#[cfg(test)]
mod test;

#[cfg(all(
    target_arch = "wasm32",
    target_vendor = "unknown",
    target_os = "unknown",
    target_env = ""
))]
mod wasm;

use json::{number::Number as JNumber, object::Object, Array, JsonValue};
use std::collections::HashMap;

pub use codegen::*;
pub use json;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Number {
    Int,
    Float,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum JsonType {
    Null,
    Number(Number),
    Boolean,
    String,
    Object(String),
    Array(Box<JsonType>),
}

pub trait TypeAccumulator {
    fn end(&mut self) -> String;

    fn number(&mut self, key: &str, number: Number) -> Result<(), Error>;
    fn boolean(&mut self, key: &str) -> Result<(), Error>;
    fn string(&mut self, key: &str) -> Result<(), Error>;
    fn unknown(&mut self, key: &str) -> Result<(), Error>;
    fn array(&mut self, key: &str, ty: JsonType) -> Result<(), Error>;
    fn object(&mut self, key: &str, object_name: &str) -> Result<(), Error>;

    fn push_object_type(&mut self, object_name: &str) -> Result<(), Error>;
    fn pop_object_type(&mut self) -> Result<(), Error>;

    fn prefered_object_name(&self) -> String;
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Got a parse error from the [`json`] crate.
    Parse(json::Error),
    /// All inputs should begin as an object `{ ... }`.
    ExpectedObject,
    /// A [`json`] number does not fit within a [`f64`] or [`i64`].
    /// This error does not seem to work.
    BadNumber,
    /// A type is not supported by the choosen language.
    TypeNotSupported,
    /// An array cannot contain multiple elements of differing types.
    DifferingArrayType,
}

/// Goes with [`ObjectTypeTable`] to decrease ambiguity.
type ObjectField = String;

/// Allow prevent similar json objects from creating separate types.
#[derive(Clone, Default)]
struct ObjectTypeTable(HashMap<Vec<(ObjectField, JsonType)>, String>);

impl ObjectTypeTable {
    /// `fields` can be unsorted.
    pub fn get_object_name(&self, mut fields: Vec<(ObjectField, JsonType)>) -> Option<String> {
        fields.sort_by(|a, b| a.0.cmp(&b.0));
        self.0.get(&fields).cloned()
    }

    /// `fields` can be unsorted.
    pub fn insert(&mut self, object_name: &str, mut fields: Vec<(ObjectField, JsonType)>) {
        fields.sort_by(|a, b| a.0.cmp(&b.0));
        self.0.insert(fields, String::from(object_name));
    }

    pub fn count(&self) -> usize {
        self.0.len()
    }
}

/// Generate language bindings using a provided accumulator.
/// See the [`codegen`] for supported accumulators or build your own.
pub fn generate(
    accumulator: &mut Box<dyn TypeAccumulator>,
    name: &str,
    json_str: &str,
) -> Result<String, Error> {
    let val = json::parse(json_str).map_err(Error::Parse)?;

    let mut obj_table = ObjectTypeTable::default();

    match val {
        JsonValue::Object(val) => {
            accumulator.push_object_type(name)?;
            generate_object(accumulator, &mut obj_table, &val)?;
            accumulator.pop_object_type()?;
        }
        _ => Err(Error::ExpectedObject)?,
    };

    Ok(accumulator.end())
}

fn generate_object(
    accumulator: &mut Box<dyn TypeAccumulator>,
    obj_table: &mut ObjectTypeTable,
    val: &Object,
) -> Result<(), Error> {
    let res = val.iter().try_for_each(|(name, val)| match val {
        JsonValue::Null => accumulator.unknown(name),
        JsonValue::Short(_) | JsonValue::String(_) => accumulator.string(name),
        JsonValue::Number(n) => {
            let number = value_into_number(n)?;
            accumulator.number(name, number)
        }
        JsonValue::Boolean(_) => accumulator.boolean(name),
        JsonValue::Array(a) => {
            let ty = get_array_element_type(accumulator, obj_table, name, a)?;
            accumulator.array(name, ty)
        }
        JsonValue::Object(o) => {
            let object_name = get_object_type(accumulator, obj_table, name, o)?;
            accumulator.object(name, &object_name)
        }
    });
    res
}

fn get_array_element_type(
    accumulator: &mut Box<dyn TypeAccumulator>,
    obj_table: &mut ObjectTypeTable,
    key: &str,
    val: &Array,
) -> Result<JsonType, Error> {
    let Some(overall_val) =  val.get(0) else {
        return Ok(JsonType::Null);
    };
    let overall_type = value_into_json_type(accumulator, obj_table, key, overall_val)?;
    val.iter().try_for_each(|val| {
        if overall_type == value_into_json_type(accumulator, obj_table, key, val)? {
            Ok(())
        } else {
            Err(Error::DifferingArrayType)
        }
    })?;
    Ok(overall_type)
}

fn value_into_number(n: &JNumber) -> Result<Number, Error> {
    let n = *n;
    if i64::try_from(n).is_ok() {
        Ok(Number::Int)
    } else if f64::try_from(n).is_ok() {
        Ok(Number::Float)
    } else {
        Err(Error::BadNumber)
    }
}

fn get_object_type(
    accumulator: &mut Box<dyn TypeAccumulator>,
    obj_table: &mut ObjectTypeTable,
    key: &str,
    val: &Object,
) -> Result<String, Error> {
    let object_fields = object_into_fields(accumulator, obj_table, key, val)?;
    if let Some(name) = obj_table.get_object_name(object_fields.clone()) {
        Ok(name)
    } else {
        let name = accumulator.prefered_object_name() + &obj_table.count().to_string();
        accumulator.push_object_type(&name)?;
        generate_object(accumulator, obj_table, val)?;
        accumulator.pop_object_type()?;
        obj_table.insert(&name, object_fields);
        Ok(name)
    }
}

/// Note that this calls [`generate_object`] if the type is an object that has not yet been generated.
fn value_into_json_type(
    accumulator: &mut Box<dyn TypeAccumulator>,
    obj_table: &mut ObjectTypeTable,
    key: &str,
    val: &JsonValue,
) -> Result<JsonType, Error> {
    Ok(match val {
        JsonValue::Null => JsonType::Null,
        JsonValue::Short(_) | JsonValue::String(_) => JsonType::String,
        JsonValue::Number(n) => JsonType::Number(value_into_number(n)?),
        JsonValue::Boolean(_) => JsonType::Boolean,
        JsonValue::Object(o) => JsonType::Object(get_object_type(accumulator, obj_table, key, o)?),
        JsonValue::Array(a) => JsonType::Array(Box::new(get_array_element_type(
            accumulator,
            obj_table,
            key,
            a,
        )?)),
    })
}

fn object_into_fields(
    accumulator: &mut Box<dyn TypeAccumulator>,
    obj_table: &mut ObjectTypeTable,
    key_name: &str,
    obj: &Object,
) -> Result<Vec<(ObjectField, JsonType)>, Error> {
    obj.iter()
        .map(|(key, val)| {
            Ok((
                key.to_owned(),
                value_into_json_type(accumulator, obj_table, key_name, val)?,
            ))
        })
        .collect::<Result<_, _>>()
}
