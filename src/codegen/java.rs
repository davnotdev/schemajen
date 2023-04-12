use super::*;

#[derive(Default)]
pub struct JavaAccumulator {
    use_unknown: bool,
    struct_stack: Vec<String>,
    done_list: Vec<String>,
}

impl JavaAccumulator {
    pub fn begin() -> Self {
        JavaAccumulator::default()
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
            JsonType::Boolean => String::from("boolean"),
            JsonType::String => String::from("String"),
            JsonType::Object(ty) => ty,
            JsonType::Array(ty) => format!("java.util.Vector<{}>", self.get_type(*ty)),
        }
    }

    fn get_number(&self, num: Number) -> &'static str {
        match num {
            Number::Int => "int",
            Number::Float => "float",
        }
    }
}

impl TypeAccumulator for JavaAccumulator {
    fn end(&mut self) -> String {
        let mut end_str = String::new();

        if self.use_unknown {
            end_str += "public class Unknown implements java.io.Serializable {}\n\n"
        }

        self.done_list.iter().for_each(|done| end_str += done);

        end_str
    }

    fn number(&mut self, key: &str, number: Number) -> Result<(), Error> {
        let num_ty = self.get_number(number);
        let acc = self.get_current();
        *acc += &format!("\tpublic {} {};\n", num_ty, key);
        Ok(())
    }

    fn boolean(&mut self, key: &str) -> Result<(), Error> {
        let acc = self.get_current();
        *acc += &format!("\tpublic boolean {};\n", key);
        Ok(())
    }

    fn string(&mut self, key: &str) -> Result<(), Error> {
        let acc = self.get_current();
        *acc += &format!("\tpublic String {};\n", key);
        Ok(())
    }

    fn unknown(&mut self, key: &str) -> Result<(), Error> {
        self.use_unknown = true;
        let acc = self.get_current();
        *acc += &format!("\tpublic Unknown {};\n", key);
        Ok(())
    }

    fn array(&mut self, key: &str, ty: JsonType) -> Result<(), Error> {
        let ty = self.get_type(ty);
        let acc = self.get_current();
        *acc += &format!("\tpublic java.util.Vector<{}> {};\n", ty, key);
        Ok(())
    }

    fn object(&mut self, key: &str, object_name: &str) -> Result<(), Error> {
        let acc = self.get_current();
        *acc += &format!("\tpublic {} {};\n", object_name, key);
        Ok(())
    }

    fn push_object_type(&mut self, object_name: &str) -> Result<(), Error> {
        self.struct_stack.push(String::new());
        let acc = self.get_current();
        *acc += &format!(
            "public class {} implements java.io.Serializable {{\n",
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
