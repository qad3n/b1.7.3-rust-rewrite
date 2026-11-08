pub struct DateFields
{
    pub m_year: i64,
    pub m_month: i32,
    pub m_day: i32,
    pub m_hour: i32,
    pub m_minute: i32,
    pub m_second: i32,
    pub m_dayOfWeek: i32,
    pub m_offsetMillis: i64,
}

fn localOffsetMillis(millis: i64) -> i64
{
    use chrono::{Local, Offset, TimeZone};
    match chrono::DateTime::from_timestamp_millis(millis)
    {
        Some(instant) => Local.offset_from_utc_datetime(&instant.naive_utc()).fix().local_minus_utc() as i64 * 1000,
        None => 0,
    }
}

pub fn dateFields(utcMillis: i64) -> DateFields
{
    let millis: i64 = utcMillis + localOffsetMillis(utcMillis);
    let days: i64 = millis.div_euclid(86_400_000);
    let millisOfDay: i64 = millis.rem_euclid(86_400_000);
    let z: i64 = days + 719_468;
    let era: i64 = z.div_euclid(146_097);
    let dayOfEra: i64 = z - era * 146_097;
    let yearOfEra: i64 = (dayOfEra - dayOfEra / 1460 + dayOfEra / 36524 - dayOfEra / 146_096) / 365;
    let dayOfYear: i64 = dayOfEra - (365 * yearOfEra + yearOfEra / 4 - yearOfEra / 100);
    let monthIndex: i64 = (5 * dayOfYear + 2) / 153;
    let day: i64 = dayOfYear - (153 * monthIndex + 2) / 5 + 1;
    let month: i64 = if monthIndex < 10
    {
        monthIndex + 3
    }
    else
    {
        monthIndex - 9
    };
    let year: i64 = yearOfEra + era * 400 + if month <= 2 { 1 } else { 0 };
    DateFields {
        m_year: year,
        m_month: month as i32,
        m_day: day as i32,
        m_hour: (millisOfDay / 3_600_000) as i32,
        m_minute: (millisOfDay / 60_000 % 60) as i32,
        m_second: (millisOfDay / 1000 % 60) as i32,
        m_dayOfWeek: (days + 4).rem_euclid(7) as i32,
        m_offsetMillis: millis - utcMillis,
    }
}
