use super::*;

pub struct MockAccumulator(String);

impl MockAccumulator {
    pub fn begin() -> Self {
        MockAccumulator(String::new())
    }
}

impl TypeAccumulator for MockAccumulator {
    fn end(&mut self) -> String {
        self.0.clone()
    }

    fn number(&mut self, key: &str, number: Number) -> Result<(), Error> {
        self.0 += &format!("num:{}:{:?}\n", key, number);
        Ok(())
    }

    fn boolean(&mut self, key: &str) -> Result<(), Error> {
        self.0 += &format!("bool:{}\n", key);
        Ok(())
    }

    fn string(&mut self, key: &str) -> Result<(), Error> {
        self.0 += &format!("str:{}\n", key);
        Ok(())
    }

    fn unknown(&mut self, key: &str) -> Result<(), Error> {
        self.0 += &format!("null:{}\n", key);
        Ok(())
    }

    fn array(&mut self, key: &str, ty: JsonType) -> Result<(), Error> {
        self.0 += &format!("arr:{}:{:?}\n", key, ty);
        Ok(())
    }

    fn object(&mut self, key: &str, object_name: &str) -> Result<(), Error> {
        self.0 += &format!("obj:{}:{}\n", key, object_name);
        Ok(())
    }

    fn push_object_type(&mut self, object_name: &str) -> Result<(), Error> {
        self.0 += &format!("ty:{}\n", object_name);
        Ok(())
    }

    fn pop_object_type(&mut self) -> Result<(), Error> {
        self.0 += "popty\n";
        Ok(())
    }

    fn prefered_object_name(&self) -> String {
        String::from("_")
    }
}
