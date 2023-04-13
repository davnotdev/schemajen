use super::*;

#[derive(Default)]
pub struct TypescriptAccumulator {
    struct_stack: Vec<String>,
    done_list: Vec<String>,
}

impl TypescriptAccumulator {
    pub fn begin() -> Self {
        TypescriptAccumulator::default()
    }

    fn get_current(&mut self) -> &mut String {
        self.struct_stack.last_mut().unwrap()
    }

    fn get_type(ty: JsonType) -> String {
        match ty {
            JsonType::Null => String::from("unknown"),
            JsonType::Number(_) => String::from("number"),
            JsonType::Boolean => String::from("boolean"),
            JsonType::String => String::from("string"),
            JsonType::Object(ty) => ty,
            JsonType::Array(ty) => format!("{}[]", Self::get_type(*ty)),
        }
    }
}

impl TypeAccumulator for TypescriptAccumulator {
    fn end(&mut self) -> String {
        let mut end_str = String::new();

        self.done_list.iter().for_each(|done| end_str += done);

        end_str
    }

    fn number(&mut self, key: &str, _: Number) -> Result<(), Error> {
        let acc = self.get_current();
        *acc += &format!("\t{}: number;\n", key);
        Ok(())
    }

    fn boolean(&mut self, key: &str) -> Result<(), Error> {
        let acc = self.get_current();
        *acc += &format!("\t{}: boolean;\n", key);
        Ok(())
    }

    fn string(&mut self, key: &str) -> Result<(), Error> {
        let acc = self.get_current();
        *acc += &format!("\t{}: string;\n", key);
        Ok(())
    }

    fn unknown(&mut self, key: &str) -> Result<(), Error> {
        let acc = self.get_current();
        *acc += &format!("\t{}: unknown;\n", key);
        Ok(())
    }

    fn array(&mut self, key: &str, ty: JsonType) -> Result<(), Error> {
        let ty = Self::get_type(ty);
        let acc = self.get_current();
        *acc += &format!("\t{}: {}[];\n", key, ty);
        Ok(())
    }

    fn object(&mut self, key: &str, object_name: &str) -> Result<(), Error> {
        let acc = self.get_current();
        *acc += &format!("\t{}: {};\n", key, object_name);
        Ok(())
    }

    fn push_object_type(&mut self, object_name: &str) -> Result<(), Error> {
        self.struct_stack.push(String::new());
        let acc = self.get_current();
        *acc += &format!("type {} = {{\n", object_name);
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
