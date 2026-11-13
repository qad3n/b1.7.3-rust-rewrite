use std::net::IpAddr;

// streaming services serve their full page with a track title only to browsers
const AGENT: &str = "Mozilla/5.0";
const SCHEMES: [&str; 2] = ["https://", "http://"];
const LOCAL_HOST: &str = "localhost";
const YOUTUBE_HOSTS: [&str; 2] = ["youtube.com", "youtu.be"];
// a page title ends with its site name after one of these
const SITE_SEPARATORS: [&str; 3] = [" | ", " - ", " \u{2013} "];
// apple music starts its titles with a left to right mark and spaces its own name with a no break space
const INVISIBLE: char = '\u{200e}';
const NO_BREAK_SPACE: char = '\u{a0}';
const ENTITIES: [(&str, &str); 6] =
    [("&amp;", "&"), ("&quot;", "\""), ("&#39;", "'"), ("&#x27;", "'"), ("&lt;", "<"), ("&gt;", ">")];

pub fn isLink(text: &str) -> bool { SCHEMES.iter().any(|scheme| hasScheme(text, scheme)) }

pub fn youtube(link: &str) -> bool
{
    host(link).is_some_and(|host|
    {
        YOUTUBE_HOSTS.iter().any(|youtube| host == *youtube || host.ends_with(&format!(".{youtube}")))
    })
}

fn host(link: &str) -> Option<String>
{
    let rest: &str = SCHEMES.iter().find_map(|scheme| link.get(scheme.len()..).filter(|_| hasScheme(link, scheme)))?;
    let authority: &str = rest.split(['/', '?', '#']).next()?;
    let host: &str = authority.rsplit('@').next()?;
    let name: &str = match host.strip_prefix('[')
    {
        Some(bracketed) => bracketed.split(']').next()?,
        None => host.split(':').next()?,
    };
    Some(name.to_ascii_lowercase())
}

fn hasScheme(link: &str, scheme: &str) -> bool
{
    link.get(..scheme.len()).is_some_and(|start| start.eq_ignore_ascii_case(scheme))
}

// a web page on a public host, never a file or a machine on the server's own network
pub fn check(link: &str) -> Result<(), String>
{
    let name: String = host(link).ok_or("the link must start with https://")?;
    let local: bool = name.is_empty() || name == LOCAL_HOST || name.parse::<IpAddr>().is_ok();
    if local
    {
        return Err("the link must name a website".to_owned());
    }

    Ok(())
}

// the song a page names, from its title without the site name
pub fn title(link: &str) -> Result<String, String>
{
    let mut response: ureq::http::Response<ureq::Body> = ureq::get(link)
        .header("User-Agent", AGENT)
        .call()
        .map_err(|error| format!("could not open the link: {error}"))?;
    let html: String =
        response.body_mut().read_to_string().map_err(|error| format!("could not read the link: {error}"))?;

    let mut title: String = decoded(element(&html, "title").ok_or("the link names no song")?);
    let site: Option<String> = meta(&html, "og:site_name").map(decoded);

    // the site may name itself longer in its meta than in its title as apple music adds web player
    if let Some(site) = site
    {
        for separator in SITE_SEPARATORS
        {
            if let Some((kept, tail)) = title.rsplit_once(separator)
                && !tail.is_empty()
                && site.starts_with(tail)
            {
                title = kept.to_owned();
            }
        }
    }

    if title.trim().is_empty()
    {
        return Err("the link names no song".to_owned());
    }

    Ok(title.trim().to_owned())
}

fn element<'a>(html: &'a str, name: &str) -> Option<&'a str>
{
    let open: usize = html.find(&format!("<{name}"))?;
    let start: usize = open + html[open..].find('>')? + 1;
    let end: usize = start + html[start..].find(&format!("</{name}>"))?;
    Some(&html[start..end])
}

fn meta<'a>(html: &'a str, property: &str) -> Option<&'a str>
{
    let marker: String = format!("property=\"{property}\" content=\"");
    let start: usize = html.find(&marker)? + marker.len();
    let end: usize = start + html[start..].find('"')?;
    Some(&html[start..end])
}

fn decoded(text: &str) -> String
{
    let mut text: String = text.replace(INVISIBLE, "").replace(NO_BREAK_SPACE, " ");
    for (entity, plain) in ENTITIES
    {
        text = text.replace(entity, plain);
    }
    text.trim().to_owned()
}
