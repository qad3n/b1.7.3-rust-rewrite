use jvm::string::JString;

pub trait JsonListener
{
    fn startDocument(&mut self);

    fn endDocument(&mut self);

    fn startArray(&mut self);

    fn endArray(&mut self);

    fn startObject(&mut self);

    fn endObject(&mut self);

    fn startField(&mut self, name: JString);

    fn endField(&mut self);

    fn stringValue(&mut self, value: JString);

    fn numberValue(&mut self, value: JString);

    fn trueValue(&mut self);

    fn falseValue(&mut self);

    fn nullValue(&mut self);
}
