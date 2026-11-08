use crate::port::decimal_format;

pub trait IStatType
{
    fn format(&self, value: i32) -> String;
}

pub struct StatTypeSimple;

pub struct StatTypeTime;

pub struct StatTypeDistance;

impl IStatType for StatTypeSimple
{
    // port decimal_format stands in for java DecimalFormat
    fn format(&self, value: i32) -> String { decimal_format::formatInteger(value) }
}

impl IStatType for StatTypeTime
{
    fn format(&self, value: i32) -> String
    {
        let seconds: f64 = value as f64 / 20.0;
        let minutes: f64 = seconds / 60.0;
        let hours: f64 = minutes / 60.0;
        let days: f64 = hours / 24.0;
        let years: f64 = days / 365.0;
        if years > 0.5
        {
            // port decimal_format stands in for java DecimalFormat
            return format!("{} y", decimal_format::formatTwoPlaces(years));
        }
        if days > 0.5
        {
            // port decimal_format stands in for java DecimalFormat
            return format!("{} d", decimal_format::formatTwoPlaces(days));
        }
        if hours > 0.5
        {
            // port decimal_format stands in for java DecimalFormat
            return format!("{} h", decimal_format::formatTwoPlaces(hours));
        }
        if minutes > 0.5
        {
            // port decimal_format stands in for java DecimalFormat
            return format!("{} m", decimal_format::formatTwoPlaces(minutes));
        }
        format!("{} s", jvm::floating_decimal::doubleToString(seconds))
    }
}

impl IStatType for StatTypeDistance
{
    fn format(&self, value: i32) -> String
    {
        let meters: f64 = value as f64 / 100.0;
        let kilometers: f64 = meters / 1000.0;
        if kilometers > 0.5
        {
            // port decimal_format stands in for java DecimalFormat
            return format!("{} km", decimal_format::formatTwoPlaces(kilometers));
        }
        if meters > 0.5
        {
            // port decimal_format stands in for java DecimalFormat
            return format!("{} m", decimal_format::formatTwoPlaces(meters));
        }
        format!("{value} cm")
    }
}
