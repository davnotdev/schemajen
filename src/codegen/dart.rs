use super::*;

pub struct ArrayField {
    very_inner: String,
    depth: usize,
}

impl ArrayField {
    pub fn depth_from_ty(ty: &JsonType, begin: usize) -> usize {
        let depth = begin;
        match ty {
            JsonType::Array(a) => Self::depth_from_ty(a, depth + 1),
            _ => depth,
        }
    }
}

enum FieldType {
    Primitive,
    Struct,
    Array(ArrayField),
}

struct StructItem {
    acc: String,
    name: String,
    fields: Vec<(String, FieldType)>,
}

#[derive(Default)]
pub struct DartAccumulator {
    struct_stack: Vec<StructItem>,
    done_list: Vec<String>,
}

impl DartAccumulator {
    pub fn begin() -> Self {
        DartAccumulator::default()
    }

    fn get_current(&mut self) -> &mut StructItem {
        self.struct_stack.last_mut().unwrap()
    }

    fn is_builtin_type(s: &str) -> bool {
        matches!(s, "dynamic" | "bool" | "String" | "double" | "int")
    }

    fn get_type(&mut self, ty: JsonType) -> String {
        match ty {
            JsonType::Null => String::from("dynamic"),
            JsonType::Number(n) => String::from(self.get_number(n)),
            JsonType::Boolean => String::from("bool"),
            JsonType::String => String::from("String"),
            JsonType::Object(ty) => ty,
            JsonType::Array(ty) => format!("List<{}>", self.get_type(*ty)),
        }
    }

    fn get_without_outer_list(&mut self, ty: JsonType) -> String {
        match ty {
            JsonType::Null => String::from("dynamic"),
            JsonType::Number(n) => String::from(self.get_number(n)),
            JsonType::Boolean => String::from("bool"),
            JsonType::String => String::from("String"),
            JsonType::Object(ty) => ty,
            JsonType::Array(ty) => self.get_without_outer_list(*ty),
        }
    }

    fn get_number(&self, num: Number) -> &'static str {
        match num {
            Number::Int => "int",
            Number::Float => "double",
        }
    }
}

impl TypeAccumulator for DartAccumulator {
    fn end(&mut self) -> String {
        let mut end_str = String::new();

        self.done_list.iter().for_each(|done| end_str += done);

        end_str += r#"//  import 'dart:convert';
//  var data = jsonDecode("{ ... }");
//  var ty = T.fromJson(data);
//  jsonEncode(ty.toJson());"#;

        end_str
    }

    fn number(&mut self, key: &str, number: Number) -> Result<(), Error> {
        let num_ty = self.get_number(number);
        let acc = self.get_current();
        acc.acc += &format!("\tfinal {} {};\n", num_ty, key);
        acc.fields.push((String::from(key), FieldType::Primitive));
        Ok(())
    }

    fn boolean(&mut self, key: &str) -> Result<(), Error> {
        let acc = self.get_current();
        acc.acc += &format!("\tfinal bool {};\n", key);
        acc.fields.push((String::from(key), FieldType::Primitive));
        Ok(())
    }

    fn string(&mut self, key: &str) -> Result<(), Error> {
        let acc = self.get_current();
        acc.acc += &format!("\tfinal String {};\n", key);
        acc.fields.push((String::from(key), FieldType::Primitive));
        Ok(())
    }

    fn unknown(&mut self, key: &str) -> Result<(), Error> {
        let acc = self.get_current();
        acc.acc += &format!("\tfinal dynamic {};\n", key);
        acc.fields.push((String::from(key), FieldType::Primitive));
        Ok(())
    }

    fn array(&mut self, key: &str, ty: JsonType) -> Result<(), Error> {
        let ty_str = self.get_type(ty.clone());
        let ty_without_outer = self.get_without_outer_list(ty.clone());
        let acc = self.get_current();
        let ty_name = format!("List<{}>", ty_str);
        acc.acc += &format!("\tfinal {} {};\n", ty_name, key);
        acc.fields.push((
            String::from(key),
            FieldType::Array(ArrayField {
                very_inner: ty_without_outer,
                depth: ArrayField::depth_from_ty(&ty, 1),
            }),
        ));
        Ok(())
    }

    fn object(&mut self, key: &str, object_name: &str) -> Result<(), Error> {
        let acc = self.get_current();
        acc.acc += &format!("\tfinal {} {};\n", object_name, key);
        acc.fields.push((String::from(key), FieldType::Struct));
        Ok(())
    }

    fn push_object_type(&mut self, object_name: &str) -> Result<(), Error> {
        self.struct_stack.push(StructItem {
            acc: String::new(),
            name: String::from(object_name),
            fields: vec![],
        });
        let acc = self.get_current();
        acc.acc += &format!("class {} {{\n", object_name);
        Ok(())
    }

    fn pop_object_type(&mut self) -> Result<(), Error> {
        let acc = self.get_current();
        acc.acc += &format!("\t{}.fromJson(Map<String, dynamic> json):\n", acc.name);
        acc.fields.iter().for_each(|(field, field_ty)| {
            acc.acc += &format!(
                "\t\t{} = {},\n",
                field,
                match field_ty {
                    FieldType::Struct | FieldType::Primitive => format!("json['{}']", field),
                    FieldType::Array(s) => {
                        fn list_from_depth(name: &str, depth: usize) -> String {
                            let mut acc = String::from(name);
                            (0..depth + 1).for_each(|_| {
                                acc = format!("List<{}>", acc);
                            });
                            acc
                        }
                        let mut iname = String::from("i");
                        let mut acc = if Self::is_builtin_type(&s.very_inner) {
                            iname.clone()
                        } else {
                            format!("{}.fromJson({})", s.very_inner, iname)
                        };
                        for depth in 0..s.depth {
                            let next_iname = if depth == s.depth - 1 {
                                format!("json['{}']", field)
                            } else {
                                format!("i{}", depth)
                            };

                            acc = format!(
                                "{}.from({}.map(({}) => {}))",
                                list_from_depth(&s.very_inner, depth),
                                next_iname,
                                iname,
                                acc
                            );
                            iname = next_iname;
                        }
                        acc
                    }
                }
            );
        });
        acc.acc.pop();
        acc.acc.pop();
        acc.acc += ";\n\n";
        acc.acc += "\tMap<String, dynamic> toJson() => {";
        acc.fields.iter().for_each(|(field, field_ty)| {
            acc.acc += &format!(
                "\n\t\t'{}': {},",
                field,
                &match field_ty {
                    FieldType::Struct => field.to_owned() + ".toJson()",
                    FieldType::Primitive => field.to_owned(),
                    FieldType::Array(s) => {
                        let mut iname = String::from("i");
                        let mut acc = if Self::is_builtin_type(&s.very_inner) {
                            iname.clone()
                        } else {
                            iname.clone() + ".toJson()"
                        };
                        for depth in 0..s.depth {
                            let next_iname = if depth == s.depth - 1 {
                                field.to_owned()
                            } else {
                                format!("i{}", depth)
                            };
                            acc = format!("{}.map(({}) => {}).toList()", next_iname, iname, acc);
                            iname = next_iname;
                        }
                        acc
                    }
                }
            );
        });
        acc.acc += "\n\t};\n";
        acc.acc += "}\n\n";
        let s = self.struct_stack.pop().unwrap();
        self.done_list.push(s.acc);
        Ok(())
    }

    fn prefered_object_name(&self) -> String {
        String::from("_Type")
    }
}
