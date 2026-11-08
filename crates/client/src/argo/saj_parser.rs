use crate::argo::{InvalidSyntaxException, JsonListener, PositionTrackingPushbackReader};
use jvm::string::JString;
use jvm::throwable::Throwable;

pub struct SajParser;

type ParseResult<T> = Result<T, Throwable>;

fn show(c: u16) -> String { String::from_utf16_lossy(&[c]) }

fn arrayToString(chars: &[u16]) -> String
{
    format!("[{}]", chars.iter().map(|c| show(*c)).collect::<Vec<String>>().join(", "))
}

fn syntax(reader: &PositionTrackingPushbackReader, message: String) -> Throwable
{
    InvalidSyntaxException::new(&message, reader)
}

impl SajParser
{
    pub fn parse(&self, input: &JString, jsonListener: &mut dyn JsonListener) -> ParseResult<()>
    {
        let mut reader: PositionTrackingPushbackReader = PositionTrackingPushbackReader::new(input);
        let c: u16 = reader.read() as u16;
        match c
        {
            0x7B =>
            {
                reader.unread(c);
                jsonListener.startDocument();
                self.objectString(&mut reader, jsonListener)?;
            }
            0x5B =>
            {
                reader.unread(c);
                jsonListener.startDocument();
                self.arrayString(&mut reader, jsonListener)?;
            }
            _ => return Err(syntax(&reader, format!("Expected either [ or {{ but got [{}].", show(c)))),
        }
        let n: i32 = self.readNextNonWhitespaceChar(&mut reader);
        if n != -1
        {
            return Err(syntax(&reader, format!("Got unexpected trailing character [{}].", show(n as u16))));
        }
        jsonListener.endDocument();
        Ok(())
    }

    fn arrayString(
        &self,
        reader: &mut PositionTrackingPushbackReader,
        jsonListener: &mut dyn JsonListener,
    ) -> ParseResult<()>
    {
        let c0: u16 = self.readNextNonWhitespaceChar(reader) as u16;
        if c0 != 0x5B
        {
            return Err(syntax(reader, format!("Expected object to start with [ but got [{}].", show(c0))));
        }
        jsonListener.startArray();
        let c1: u16 = self.readNextNonWhitespaceChar(reader) as u16;
        reader.unread(c1);
        if c1 != 0x5D
        {
            self.aJsonValue(reader, jsonListener)?;
        }
        loop
        {
            let c2: u16 = self.readNextNonWhitespaceChar(reader) as u16;
            match c2
            {
                0x2C => self.aJsonValue(reader, jsonListener)?,
                0x5D => break,
                _ => return Err(syntax(reader, format!("Expected either , or ] but got [{}].", show(c2)))),
            }
        }
        jsonListener.endArray();
        Ok(())
    }

    fn objectString(
        &self,
        reader: &mut PositionTrackingPushbackReader,
        jsonListener: &mut dyn JsonListener,
    ) -> ParseResult<()>
    {
        let c0: u16 = self.readNextNonWhitespaceChar(reader) as u16;
        if c0 != 0x7B
        {
            return Err(syntax(reader, format!("Expected object to start with {{ but got [{}].", show(c0))));
        }
        jsonListener.startObject();
        let c1: u16 = self.readNextNonWhitespaceChar(reader) as u16;
        reader.unread(c1);
        if c1 != 0x7D
        {
            self.aFieldToken(reader, jsonListener)?;
        }
        loop
        {
            let c2: u16 = self.readNextNonWhitespaceChar(reader) as u16;
            match c2
            {
                0x2C => self.aFieldToken(reader, jsonListener)?,
                0x7D => break,
                _ => return Err(syntax(reader, format!("Expected either , or }} but got [{}].", show(c2)))),
            }
        }
        jsonListener.endObject();
        Ok(())
    }

    fn aFieldToken(
        &self,
        reader: &mut PositionTrackingPushbackReader,
        jsonListener: &mut dyn JsonListener,
    ) -> ParseResult<()>
    {
        let c: u16 = self.readNextNonWhitespaceChar(reader) as u16;
        if c != 0x22
        {
            return Err(syntax(
                reader,
                format!("Expected object identifier to begin with [\"] but got [{}].", show(c)),
            ));
        }
        reader.unread(c);
        let name: JString = self.stringToken(reader)?;
        jsonListener.startField(name);
        let c2: u16 = self.readNextNonWhitespaceChar(reader) as u16;
        if c2 != 0x3A
        {
            return Err(syntax(
                reader,
                format!("Expected object identifier to be followed by : but got [{}].", show(c2)),
            ));
        }
        self.aJsonValue(reader, jsonListener)?;
        jsonListener.endField();
        Ok(())
    }

    fn literalTail(&self, reader: &mut PositionTrackingPushbackReader, first: char, rest: &str) -> ParseResult<()>
    {
        let expected: Vec<u16> = rest.encode_utf16().collect();
        let mut cArray: Vec<u16> = vec![0; expected.len()];
        let n: i32 = reader.readBuffer(&mut cArray);
        if n != expected.len() as i32 || cArray != expected
        {
            reader.uncount(&cArray);
            let wanted: Vec<String> = rest.chars().map(String::from).collect();
            return Err(syntax(
                reader,
                format!(
                    "Expected '{first}' to be followed by [[{}]], but got [{}].",
                    wanted.join(", "),
                    arrayToString(&cArray)
                ),
            ));
        }
        Ok(())
    }

    fn aJsonValue(
        &self,
        reader: &mut PositionTrackingPushbackReader,
        jsonListener: &mut dyn JsonListener,
    ) -> ParseResult<()>
    {
        let c: u16 = self.readNextNonWhitespaceChar(reader) as u16;
        match c
        {
            0x22 =>
            {
                reader.unread(c);
                let value: JString = self.stringToken(reader)?;
                jsonListener.stringValue(value);
            }
            0x74 =>
            {
                self.literalTail(reader, 't', "rue")?;
                jsonListener.trueValue();
            }
            0x66 =>
            {
                self.literalTail(reader, 'f', "alse")?;
                jsonListener.falseValue();
            }
            0x6E =>
            {
                self.literalTail(reader, 'n', "ull")?;
                jsonListener.nullValue();
            }
            0x2D | 0x30..=0x39 =>
            {
                reader.unread(c);
                let value: JString = self.numberToken(reader)?;
                jsonListener.numberValue(value);
            }
            0x7B =>
            {
                reader.unread(c);
                self.objectString(reader, jsonListener)?;
            }
            0x5B =>
            {
                reader.unread(c);
                self.arrayString(reader, jsonListener)?;
            }
            _ => return Err(syntax(reader, format!("Invalid character at start of value [{}].", show(c)))),
        }
        Ok(())
    }

    fn numberToken(&self, reader: &mut PositionTrackingPushbackReader) -> ParseResult<JString>
    {
        let mut builder: Vec<u16> = Vec::new();
        let c: u16 = reader.read() as u16;
        if c == 0x2D
        {
            builder.push(0x2D);
        }
        else
        {
            reader.unread(c);
        }
        builder.extend(self.nonNegativeNumberToken(reader)?);
        Ok(JString::fromUtf16(builder))
    }

    fn nonNegativeNumberToken(&self, reader: &mut PositionTrackingPushbackReader) -> ParseResult<Vec<u16>>
    {
        let mut builder: Vec<u16> = Vec::new();
        let c: u16 = reader.read() as u16;
        if c == 0x30
        {
            builder.push(0x30);
        }
        else
        {
            reader.unread(c);
            builder.push(self.nonZeroDigitToken(reader)?);
            builder.extend(self.digitString(reader));
        }
        builder.extend(self.possibleFractionalComponent(reader)?);
        builder.extend(self.possibleExponent(reader)?);
        Ok(builder)
    }

    fn nonZeroDigitToken(&self, reader: &mut PositionTrackingPushbackReader) -> ParseResult<u16>
    {
        let c2: u16 = reader.read() as u16;
        if !(0x31..=0x39).contains(&c2)
        {
            return Err(syntax(reader, format!("Expected a digit 1 - 9 but got [{}].", show(c2))));
        }
        Ok(c2)
    }

    fn digitToken(&self, reader: &mut PositionTrackingPushbackReader) -> ParseResult<u16>
    {
        let c2: u16 = reader.read() as u16;
        if !(0x30..=0x39).contains(&c2)
        {
            return Err(syntax(reader, format!("Expected a digit 1 - 9 but got [{}].", show(c2))));
        }
        Ok(c2)
    }

    fn digitString(&self, reader: &mut PositionTrackingPushbackReader) -> Vec<u16>
    {
        let mut builder: Vec<u16> = Vec::new();
        loop
        {
            let c0: u16 = reader.read() as u16;
            if !(0x30..=0x39).contains(&c0)
            {
                reader.unread(c0);
                return builder;
            }
            builder.push(c0);
        }
    }

    fn possibleFractionalComponent(&self, reader: &mut PositionTrackingPushbackReader) -> ParseResult<Vec<u16>>
    {
        let mut builder: Vec<u16> = Vec::new();
        let c: u16 = reader.read() as u16;
        if c == 0x2E
        {
            builder.push(0x2E);
            builder.push(self.digitToken(reader)?);
            builder.extend(self.digitString(reader));
        }
        else
        {
            reader.unread(c);
        }
        Ok(builder)
    }

    fn possibleExponent(&self, reader: &mut PositionTrackingPushbackReader) -> ParseResult<Vec<u16>>
    {
        let mut builder: Vec<u16> = Vec::new();
        let c: u16 = reader.read() as u16;
        if c == 0x2E || c == 0x45
        {
            builder.push(0x45);
            builder.extend(self.possibleSign(reader));
            builder.push(self.digitToken(reader)?);
            builder.extend(self.digitString(reader));
        }
        else
        {
            reader.unread(c);
        }
        Ok(builder)
    }

    fn possibleSign(&self, reader: &mut PositionTrackingPushbackReader) -> Vec<u16>
    {
        let c: u16 = reader.read() as u16;
        if c == 0x2B || c == 0x2D
        {
            return vec![c];
        }
        reader.unread(c);
        Vec::new()
    }

    fn stringToken(&self, reader: &mut PositionTrackingPushbackReader) -> ParseResult<JString>
    {
        let c0: u16 = reader.read() as u16;
        if c0 != 0x22
        {
            return Err(syntax(reader, format!("Expected [\"] but got [{}].", show(c0))));
        }
        let mut builder: Vec<u16> = Vec::new();
        loop
        {
            let c1: u16 = reader.read() as u16;
            match c1
            {
                0x22 => return Ok(JString::fromUtf16(builder)),
                0x5C => builder.push(self.escapedStringChar(reader)?),
                _ => builder.push(c1),
            }
        }
    }

    fn escapedStringChar(&self, reader: &mut PositionTrackingPushbackReader) -> ParseResult<u16>
    {
        let c2: u16 = reader.read() as u16;
        let c: u16 = match c2
        {
            0x22 | 0x5C | 0x2F => c2,
            0x62 => 0x08,
            0x66 => 0x0C,
            0x6E => 0x0A,
            0x72 => 0x0D,
            0x74 => 0x09,
            0x75 => self.hexadecimalNumber(reader)? as u16,
            _ => return Err(syntax(reader, format!("Unrecognised escape character [{}].", show(c2)))),
        };
        Ok(c)
    }

    fn hexadecimalNumber(&self, reader: &mut PositionTrackingPushbackReader) -> ParseResult<i32>
    {
        let mut cArray: Vec<u16> = vec![0; 4];
        let n2: i32 = reader.readBuffer(&mut cArray);
        if n2 != 4
        {
            if n2 < 0
            {
                jvm::throwable::throw(&jvm::throwable::STRING_INDEX, &format!("String index out of range: {n2}"));
            }
            let got: String = String::from_utf16_lossy(&cArray[..n2 as usize]);
            return Err(syntax(
                reader,
                format!("Expected a 4 digit hexidecimal number but got only [{n2}], namely [{got}]."),
            ));
        }
        let text: String = String::from_utf16_lossy(&cArray);
        match jvm::integer::parseIntRadix(&JString::fromUtf16(cArray.clone()), 16)
        {
            Ok(value) => Ok(value),
            _ =>
            {
                reader.uncount(&cArray);
                Err(syntax(reader, format!("Unable to parse [{text}] as a hexidecimal number.")))
            }
        }
    }

    fn readNextNonWhitespaceChar(&self, reader: &mut PositionTrackingPushbackReader) -> i32
    {
        loop
        {
            let n: i32 = reader.read();
            if !matches!(n, 9 | 10 | 13 | 32)
            {
                return n;
            }
        }
    }
}
