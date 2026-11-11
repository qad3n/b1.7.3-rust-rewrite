use crate::AuthResult;

const FIELD_LIMIT: usize = 1 << 16;

pub(crate) struct Writer
{
    m_buf: Vec<u8>,
}

impl Writer
{
    pub(crate) fn new() -> Self { Self { m_buf: Vec::new() } }

    pub(crate) fn byte(&mut self, value: u8) { self.m_buf.push(value); }

    pub(crate) fn long(&mut self, value: u64) { self.m_buf.extend_from_slice(&value.to_be_bytes()); }

    pub(crate) fn bytes(&mut self, value: &[u8])
    {
        self.m_buf.extend_from_slice(&(value.len() as u32).to_be_bytes());
        self.m_buf.extend_from_slice(value);
    }

    pub(crate) fn text(&mut self, value: &str) { self.bytes(value.as_bytes()); }

    pub(crate) fn finish(self) -> Vec<u8> { self.m_buf }
}

pub(crate) struct Reader<'a>
{
    m_data: &'a [u8],
    m_pos: usize,
}

impl<'a> Reader<'a>
{
    pub(crate) fn new(data: &'a [u8]) -> Self { Reader { m_data: data, m_pos: 0 } }

    fn take(&mut self, len: usize) -> AuthResult<&'a [u8]>
    {
        let end: usize = self.m_pos.checked_add(len).ok_or("field length overflow")?;
        let slice: &'a [u8] = self.m_data.get(self.m_pos..end).ok_or("truncated auth data")?;

        self.m_pos = end;
        Ok(slice)
    }

    pub(crate) fn byte(&mut self) -> AuthResult<u8> { Ok(self.take(1)?[0]) }

    pub(crate) fn long(&mut self) -> AuthResult<u64>
    {
        let bytes: [u8; 8] = self.take(8)?.try_into().map_err(|_| "truncated auth data")?;
        Ok(u64::from_be_bytes(bytes))
    }

    pub(crate) fn bytes(&mut self) -> AuthResult<Vec<u8>>
    {
        let header: [u8; 4] = self.take(4)?.try_into().map_err(|_| "truncated auth data")?;
        let len: usize = u32::from_be_bytes(header) as usize;
        if len > FIELD_LIMIT
        {
            return Err(format!("auth field of {len} bytes is over the limit"));
        }

        Ok(self.take(len)?.to_vec())
    }

    pub(crate) fn text(&mut self) -> AuthResult<String>
    {
        String::from_utf8(self.bytes()?).map_err(|_| String::from("auth text is not utf-8"))
    }

    pub(crate) fn end(&self) -> AuthResult<()>
    {
        if self.m_pos != self.m_data.len()
        {
            return Err(String::from("trailing bytes after auth data"));
        }

        Ok(())
    }
}
