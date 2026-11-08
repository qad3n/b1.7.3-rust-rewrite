use crate::argo::ThingWithPosition;
use jvm::string::JString;

pub struct PositionTrackingPushbackReader
{
    m_chars: Vec<u16>,
    m_next: usize,
    m_pushback: Option<u16>,
    m_characterCount: i32,
    m_lineCount: i32,
    m_lastCharacterWasCariageReturn: bool,
}

impl PositionTrackingPushbackReader
{
    pub fn new(reader: &JString) -> PositionTrackingPushbackReader
    {
        PositionTrackingPushbackReader {
            m_chars: reader.chars().to_vec(),
            m_next: 0,
            m_pushback: None,
            m_characterCount: 0,
            m_lineCount: 1,
            m_lastCharacterWasCariageReturn: false,
        }
    }

    pub fn unread(&mut self, c: u16)
    {
        self.m_characterCount = (self.m_characterCount - 1).max(0);
        if self.m_pushback.is_some()
        {
            jvm::throwable::throw(&jvm::throwable::IO, "Pushback buffer overflow");
        }
        self.m_pushback = Some(c);
    }

    pub fn uncount(&mut self, resultCharArray: &[u16])
    {
        self.m_characterCount = (self.m_characterCount - resultCharArray.len() as i32).max(0);
    }

    fn rawRead(&mut self) -> i32
    {
        if let Some(c) = self.m_pushback.take()
        {
            return c as i32;
        }
        match self.m_chars.get(self.m_next)
        {
            Some(c) =>
            {
                self.m_next += 1;
                *c as i32
            }
            None => -1,
        }
    }

    pub fn read(&mut self) -> i32
    {
        let n: i32 = self.rawRead();
        self.updateCharacterAndLineCounts(n);
        n
    }

    pub fn readBuffer(&mut self, buffer: &mut [u16]) -> i32
    {
        let mut off: usize = 0;
        let mut avail: i32 = 0;
        if let Some(c) = self.m_pushback.take()
        {
            buffer[0] = c;
            off = 1;
            avail = 1;
        }
        let mut n: i32 = avail;
        if off < buffer.len()
        {
            let remaining: usize = self.m_chars.len() - self.m_next;
            if remaining == 0
            {
                n = if avail == 0 { -1 } else { avail };
            }
            else
            {
                let count: usize = remaining.min(buffer.len() - off);
                buffer[off..off + count].copy_from_slice(&self.m_chars[self.m_next..self.m_next + count]);
                self.m_next += count;
                n = avail + count as i32;
            }
        }
        for c in buffer.iter()
        {
            self.updateCharacterAndLineCounts(*c as i32);
        }
        n
    }

    fn updateCharacterAndLineCounts(&mut self, result: i32)
    {
        if result == 13
        {
            self.m_characterCount = 0;
            self.m_lineCount += 1;
            self.m_lastCharacterWasCariageReturn = true;
            return;
        }
        if result == 10 && !self.m_lastCharacterWasCariageReturn
        {
            self.m_characterCount = 0;
            self.m_lineCount += 1;
        }
        else
        {
            self.m_characterCount += 1;
        }
        self.m_lastCharacterWasCariageReturn = false;
    }
}

impl ThingWithPosition for PositionTrackingPushbackReader
{
    fn getColumn(&self) -> i32 { self.m_characterCount }

    fn getRow(&self) -> i32 { self.m_lineCount }
}
