use crate::throwable::{IO, Throwable, ThrowableClass};
use std::io::{Read, Write};

pub static UNKNOWN_HOST: ThrowableClass =
    ThrowableClass { m_name: "java.net.UnknownHostException", m_parent: Some(&IO) };

const HTTP_PORT: u16 = 80;
const HEX_RADIX: u32 = 16;
const LINE_END: &[u8] = b"\r\n";
const HEAD_END: &[u8] = b"\r\n\r\n";

mod http_status
{
    pub const REDIRECTS: std::ops::Range<i32> = 300..400;
    pub const CLIENT_ERROR: i32 = 400;
    pub const NOT_FOUND: i32 = 404;
    pub const GONE: i32 = 410;
}

struct Response
{
    m_status: i32,
    m_head: String,
    m_body: Vec<u8>,
}

impl Response
{
    fn header(&self, name: &str) -> Option<String>
    {
        self.m_head.lines().find_map(|line| {
            let (key, value): (&str, &str) = line.split_once(':')?;
            key.eq_ignore_ascii_case(name).then(|| value.trim().to_owned())
        })
    }
}

fn ioException(class: &'static ThrowableClass, message: String) -> Throwable { Throwable::new(class, Some(message)) }

// http://host[:port]/path split into its parts, the stand-in speaks plain http only like the 2011 endpoints
fn splitUrl(url: &str) -> Result<(String, u16, String), Throwable>
{
    let Some(rest) = url.strip_prefix("http://")
    else
    {
        return Err(ioException(&IO, format!("unknown protocol: {url}")));
    };
    let (authority, path): (&str, &str) = match rest.find('/')
    {
        Some(index) => (&rest[..index], &rest[index..]),
        None => (rest, "/"),
    };
    let (host, port): (&str, u16) = match authority.split_once(':')
    {
        Some((host, port)) => (host, port.parse().unwrap_or(HTTP_PORT)),
        None => (authority, HTTP_PORT),
    };
    Ok((host.to_owned(), port, path.to_owned()))
}

fn decodeChunked(body: &[u8]) -> Vec<u8>
{
    let mut out: Vec<u8> = Vec::new();
    let mut rest: &[u8] = body;
    while let Some(end) = rest.windows(LINE_END.len()).position(|pair| pair == LINE_END)
    {
        let sizeLine: String = String::from_utf8_lossy(&rest[..end]).into_owned();
        let size: usize =
            usize::from_str_radix(sizeLine.split(';').next().unwrap_or("0").trim(), HEX_RADIX).unwrap_or(0);
        rest = &rest[end + LINE_END.len()..];
        if size == 0 || size > rest.len()
        {
            break;
        }
        out.extend_from_slice(&rest[..size]);
        rest = rest.get(size + LINE_END.len()..).unwrap_or(&[]);
    }
    out
}

// one GET with the request headers HttpURLConnection of jdk 6 sends
fn get(url: &str) -> Result<Response, Throwable>
{
    let (host, port, path): (String, u16, String) = splitUrl(url)?;
    let mut stream: std::net::TcpStream = match std::net::TcpStream::connect((host.as_str(), port))
    {
        Ok(stream) => stream,
        Err(error) if error.kind() == std::io::ErrorKind::Other || error.to_string().contains("lookup") =>
        {
            return Err(ioException(&UNKNOWN_HOST, host));
        }
        Err(error) => return Err(crate::io::fromStd(&error)),
    };
    let request: String = format!(
        "GET {path} HTTP/1.1\r\nUser-Agent: Java/1.6.0_26\r\nHost: {host}\r\nAccept: text/html, image/gif, image/jpeg, *; q=.2, */*; q=.2\r\nConnection: close\r\n\r\n"
    );
    stream.write_all(request.as_bytes()).map_err(|error| crate::io::fromStd(&error))?;
    let mut response: Vec<u8> = Vec::new();
    stream.read_to_end(&mut response).map_err(|error| crate::io::fromStd(&error))?;
    let headerEnd: usize =
        response.windows(HEAD_END.len()).position(|window| window == HEAD_END).unwrap_or(response.len());
    let head: String = String::from_utf8_lossy(&response[..headerEnd]).into_owned();
    let body: Vec<u8> = response.get(headerEnd + HEAD_END.len()..).unwrap_or(&[]).to_vec();
    let status: i32 = head.split_whitespace().nth(1).and_then(|code| code.parse().ok()).unwrap_or(-1);
    Ok(Response { m_status: status, m_head: head, m_body: body })
}

// URL.openStream over HttpURLConnection, an error status throws like jdk 6 getInputStream
pub fn openStream(url: &str) -> Result<Vec<u8>, Throwable>
{
    let response: Response = get(url)?;
    let status: i32 = response.m_status;
    if http_status::REDIRECTS.contains(&status)
        && let Some(location) = response.header("Location")
        && location.starts_with("http://")
    {
        return openStream(&location);
    }
    if status >= http_status::CLIENT_ERROR
    {
        if status == http_status::NOT_FOUND || status == http_status::GONE
        {
            return Err(ioException(&crate::io::FILE_NOT_FOUND, url.to_owned()));
        }
        return Err(ioException(&IO, format!("Server returned HTTP response code: {status} for URL: {url}")));
    }
    if response.header("Transfer-Encoding").is_some_and(|encoding| encoding.eq_ignore_ascii_case("chunked"))
    {
        return Ok(decodeChunked(&response.m_body));
    }
    Ok(response.m_body)
}

// HttpURLConnection.connect then getResponseCode, which reads the status line without following a redirect
pub fn getResponseCode(url: &str) -> Result<i32, Throwable> { Ok(get(url)?.m_status) }

pub fn socketAddressToString(address: &std::net::SocketAddr) -> String
{
    let host: String = match address.ip().to_canonical()
    {
        std::net::IpAddr::V4(ip) => ip.to_string(),
        std::net::IpAddr::V6(ip) =>
        {
            ip.segments().iter().map(|segment| format!("{segment:x}")).collect::<Vec<String>>().join(":")
        }
    };
    format!("/{host}:{}", address.port())
}

pub fn urlEncode(text: &str) -> String
{
    let mut out: String = String::new();
    for byte in text.bytes()
    {
        match byte
        {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'.' | b'-' | b'*' | b'_' => out.push(byte as char),
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}
