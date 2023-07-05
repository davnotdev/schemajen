use super::*;

#[derive(Default)]
pub struct KotlinAccumulator {
    use_unknown: bool,
    struct_stack: Vec<String>,
    done_list: Vec<String>,
}

impl KotlinAccumulator {
    pub fn begin() -> Self {
        KotlinAccumulator::default()
    }

    fn get_current(&mut self) -> &mut String {
        self.struct_stack.last_mut().unwrap()
    }

    fn get_type(&mut self, ty: JsonType) -> String {
        match ty {
            JsonType::Null => {
                self.use_unknown = true;
                String::from("Unknown")
            }
            JsonType::Number(n) => String::from(self.get_number(n)),
            JsonType::Boolean => String::from("Boolean"),
            JsonType::String => String::from("String"),
            JsonType::Object(ty) => ty,
            JsonType::Array(ty) => format!("Array<{}>", self.get_type(*ty)),
        }
    }

    fn get_number(&self, num: Number) -> &'static str {
        match num {
            Number::Int => "Int",
            Number::Float => "Double",
        }
    }
}

impl TypeAccumulator for KotlinAccumulator {
    fn end(&mut self) -> String {
        let mut end_str = String::new();

        if self.use_unknown {
            end_str += "class Unknown;\n\n"
        }

        self.done_list.iter().for_each(|done| end_str += done);

        end_str += r#"//  import kotlinx.serialization.Serializable
//  import kotlinx.serialization.json.Json
//  import kotlinx.serialization.encodeToString
//
//  val json = Json.encodeToString(T)
//  val ty = Json.decodeFromString<T>(json)"#;

        end_str
    }

    fn number(&mut self, key: &str, number: Number) -> Result<(), Error> {
        let num_ty = self.get_number(number);
        let acc = self.get_current();
        *acc += &format!("\tval {}: {},\n", key, num_ty);
        Ok(())
    }

    fn boolean(&mut self, key: &str) -> Result<(), Error> {
        let acc = self.get_current();
        *acc += &format!("\tval {}: bool,\n", key);
        Ok(())
    }

    fn string(&mut self, key: &str) -> Result<(), Error> {
        let acc = self.get_current();
        *acc += &format!("\tval {}: String,\n", key);
        Ok(())
    }

    fn unknown(&mut self, key: &str) -> Result<(), Error> {
        self.use_unknown = true;
        let acc = self.get_current();
        *acc += &format!("\tval {}: Unknown,\n", key);
        Ok(())
    }

    fn array(&mut self, key: &str, ty: JsonType) -> Result<(), Error> {
        let ty = self.get_type(ty);
        let acc = self.get_current();
        *acc += &format!("\tval {}: Array<{}>,\n", key, ty);
        Ok(())
    }

    fn object(&mut self, key: &str, object_name: &str) -> Result<(), Error> {
        let acc = self.get_current();
        *acc += &format!("\tval {}: {},\n", key, object_name);
        Ok(())
    }

    fn push_object_type(&mut self, object_name: &str) -> Result<(), Error> {
        self.struct_stack.push(String::new());
        let acc = self.get_current();
        *acc += &format!(
            "@Serializable\nclass {} {{\n",
            object_name
        );
        Ok(())
    }

    fn pop_object_type(&mut self) -> Result<(), Error> {
        let acc = self.get_current();
        *acc += "}\n\n";
        let s = self.struct_stack.pop().unwrap();
        self.done_list.push(s);
        Ok(())
    }

    fn prefered_object_name(&self) -> String {
        String::from("_Type")
    }
}
