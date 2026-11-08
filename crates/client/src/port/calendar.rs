pub use jvm::calendar::{DateFields, dateFields};

pub fn formatDefault(millis: i64) -> String
{
    let fields: DateFields = dateFields(millis);
    let hour12: i32 = if fields.m_hour % 12 == 0
    {
        12
    }
    else
    {
        fields.m_hour % 12
    };
    let marker: &str = if fields.m_hour < 12 { "AM" } else { "PM" };
    format!(
        "{}/{}/{:02} {}:{:02} {marker}",
        fields.m_month,
        fields.m_day,
        fields.m_year.rem_euclid(100),
        hour12,
        fields.m_minute
    )
}
