use jvm::string::JString;

pub struct MD5String
{
    m_salt: String,
}

impl MD5String
{
    pub fn new(salt: &str) -> MD5String { MD5String { m_salt: salt.to_string() } }

    pub fn digest(&self, s: &str) -> String
    {
        let string: String = format!("{}{s}", self.m_salt);
        let text: JString = JString::from(string.as_str());
        let bytes: Vec<u8> = text.getBytes();
        let length: usize = text.length() as usize;
        if bytes.len() < length
        {
            jvm::throwable::throw(&jvm::throwable::ILLEGAL_ARGUMENT, "Input buffer too short");
        }
        let digest: [u8; 16] = jvm::md5::digest(&bytes[..length]);
        let hex: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
        let trimmed: &str = hex.trim_start_matches('0');
        if trimmed.is_empty()
        {
            "0".to_string()
        }
        else
        {
            trimmed.to_string()
        }
    }
}
