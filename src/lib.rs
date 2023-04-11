mod codegen;

use json::{object::Object, Array, JsonValue};
use std::collections::HashMap;

pub use json;

pub enum Number {
    Int(i64),
    UInt(i64),
    Float(f64),
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum JsonType {
    Null,
    Number,
    Boolean,
    String,
    Unknown,
    Object(String),
    Array(Box<JsonType>),
}

pub trait TypeAccumulator {
    fn end(&mut self) -> String;

    fn number(&mut self, key: &str, number: Number) -> Result<(), Error>;
    fn boolean(&mut self, key: &str, b: bool) -> Result<(), Error>;
    fn string(&mut self, key: &str, s: &str) -> Result<(), Error>;
    fn unknown(&mut self, key: &str) -> Result<(), Error>;
    fn array(&mut self, key: &str, ty: JsonType) -> Result<(), Error>;
    fn object(&mut self, key: &str, object_name: &str) -> Result<(), Error>;

    fn push_object_type(&mut self, object_name: &str) -> Result<String, Error>;
    fn pop_object_type(&mut self) -> Result<(), Error>;

    fn prefered_object_name(&self) -> String;
}

pub enum Error {
    /// Got a parse error from the [`json`] crate.
    Parse(json::Error),
    /// All inputs should begin as an object `{ ... }`.
    ExpectedObject,
    /// A [`json`] number does not fit within a [`f64`], [`u64`], or [`i64`].
    NumberTooLarge,
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
    accumulator: &mut impl TypeAccumulator,
    name: &str,
    json_str: &str,
) -> Result<String, Error> {
    let val = json::parse(json_str).map_err(Error::Parse)?;

    let mut obj_table = ObjectTypeTable::default();

    match val {
        JsonValue::Object(val) => generate_object(accumulator, &mut obj_table, name, &val)?,
        _ => Err(Error::ExpectedObject)?,
    };

    Ok(accumulator.end())
}

fn generate_object(
    accumulator: &mut impl TypeAccumulator,
    obj_table: &mut ObjectTypeTable,
    name: &str,
    val: &Object,
) -> Result<(), Error> {
    accumulator.push_object_type(name)?;
    let res = val.iter().try_for_each(|(name, val)| match val {
        JsonValue::Null => accumulator.unknown(name),
        JsonValue::Short(s) => accumulator.string(name, s.as_str()),
        JsonValue::String(s) => accumulator.string(name, s.as_str()),
        JsonValue::Number(n) => {
            let n = *n;
            let number = if let Ok(dec) = n.try_into() {
                Number::Float(dec)
            } else if let Ok(int) = n.try_into() {
                Number::Int(int)
            } else if let Ok(uint) = n.try_into() {
                Number::UInt(uint)
            } else {
                Err(Error::ExpectedObject)?
            };
            accumulator.number(name, number)
        }
        JsonValue::Boolean(b) => accumulator.boolean(name, *b),
        JsonValue::Array(a) => {
            let ty = get_array_element_type(accumulator, obj_table, a)?;
            accumulator.array(name, ty)
        }
        JsonValue::Object(o) => {
            let object_name = get_object_type(accumulator, obj_table, o)?;
            accumulator.object(name, &object_name)
        }
    });
    accumulator.pop_object_type()?;
    res
}

fn get_array_element_type(
    accumulator: &mut impl TypeAccumulator,
    obj_table: &mut ObjectTypeTable,
    val: &Array,
) -> Result<JsonType, Error> {
    let Some(overall_val) =  val.get(0) else {
        return Ok(JsonType::Null);
    };
    let overall_type = value_into_json_type(accumulator, obj_table, overall_val)?;
    val.iter().try_for_each(|val| {
        if overall_type == value_into_json_type(accumulator, obj_table, val)? {
            Ok(())
        } else {
            Err(Error::DifferingArrayType)
        }
    })?;
    Ok(overall_type)
}

fn get_object_type(
    accumulator: &mut impl TypeAccumulator,
    obj_table: &mut ObjectTypeTable,
    val: &Object,
) -> Result<String, Error> {
    let object_fields = object_into_fields(accumulator, obj_table, val)?;
    if let Some(name) = obj_table.get_object_name(object_fields.clone()) {
        Ok(name)
    } else {
        let name = accumulator.prefered_object_name() + &obj_table.count().to_string();
        accumulator.push_object_type(&name)?;
        accumulator.pop_object_type()?;
        obj_table.insert(&name, object_fields);
        Ok(name)
    }
}

/// Note that this calls [`generate_object`] if the type is an object that has not yet been generated.
fn value_into_json_type(
    accumulator: &mut impl TypeAccumulator,
    obj_table: &mut ObjectTypeTable,
    val: &JsonValue,
) -> Result<JsonType, Error> {
    Ok(match val {
        JsonValue::Null => JsonType::Null,
        JsonValue::Short(_) | JsonValue::String(_) => JsonType::String,
        JsonValue::Number(_) => JsonType::Number,
        JsonValue::Boolean(_) => JsonType::Boolean,
        JsonValue::Object(o) => JsonType::Object(get_object_type(accumulator, obj_table, o)?),
        JsonValue::Array(a) => {
            JsonType::Array(Box::new(get_array_element_type(accumulator, obj_table, a)?))
        }
    })
}

fn object_into_fields(
    accumulator: &mut impl TypeAccumulator,
    obj_table: &mut ObjectTypeTable,
    obj: &Object,
) -> Result<Vec<(ObjectField, JsonType)>, Error> {
    obj.iter()
        .map(|(key, val)| {
            Ok((
                key.to_owned(),
                value_into_json_type(accumulator, obj_table, val)?,
            ))
        })
        .collect::<Result<_, _>>()
}
