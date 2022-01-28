use serde::{de, Deserialize, Deserializer};
use std::{
    fmt,
    fmt::{Display, Formatter},
};
use time::{
    macros::format_description, serde::rfc3339, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset,
};

#[derive(Debug, Clone)]
pub struct Day {
    pub date: OffsetDateTime,
    pub sessions: SessionsJSON,
}

impl Display for Day {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let format = format_description!("[month repr:short] [day], [year] ([weekday repr:short])");
        write!(
            f,
            "{}\n{}",
            self.date.format(&format).unwrap(),
            self.sessions
        )
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SessionsJSON {
    pub session_list: Vec<Session>,
}

impl Display for SessionsJSON {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for session in &self.session_list {
            writeln!(f, "{}", session)?;
        }

        Ok(())
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Session {
    #[serde(deserialize_with = "deserialize_primitivedatetime")]
    pub time: PrimitiveDateTime, // In the timezone of the temple (according to text on the website)

    pub details: SessionDetails,
}

impl Display for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let format = format_description!("[hour repr:12]:[minute] [period]");
        write!(
            f,
            "{} - remaining seats: {}",
            self.time.format(&format).unwrap(),
            self.details.remaining_online_seats_available
        )
    }
}

fn deserialize_primitivedatetime<'de, D>(deserializer: D) -> Result<PrimitiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    struct JsonStringVisitor;

    impl<'de> de::Visitor<'de> for JsonStringVisitor {
        type Value = PrimitiveDateTime;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string containing a primitive date time similar to RFC 3339 but without an offset")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let format = format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]");
            PrimitiveDateTime::parse(v, &format).map_err(|e| E::custom(format!("{}", e)))
        }
    }

    deserializer.deserialize_any(JsonStringVisitor)
}

fn deserialize_time<'de, D>(deserializer: D) -> Result<Time, D::Error>
where
    D: Deserializer<'de>,
{
    struct JsonStringVisitor;

    impl<'de> de::Visitor<'de> for JsonStringVisitor {
        type Value = Time;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string containing a time in hh:mm format")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let format = format_description!("[hour]:[minute]");
            Time::parse(v, &format).map_err(|e| E::custom(format!("{}", e)))
        }
    }

    deserializer.deserialize_any(JsonStringVisitor)
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SessionDetails {
    pub remaining_online_seats_available: i32,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppointmentJSON {
    appointment_type: String,

    #[serde(with = "rfc3339")]
    appointment_date_time: OffsetDateTime,

    #[serde(deserialize_with = "deserialize_time")]
    appointment_time: Time, // For some reason the time in appointment_date_time seems to be wrong. But this parameter is right in the timezone of the temple.

    #[serde(skip)]
    pub local_offset: Option<UtcOffset>,
}

impl Display for AppointmentJSON {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let date_format = format_description!("[month repr:short] [day], [year]");
        let time_format = format_description!("[hour repr:12]:[minute] [period]");

        write!(
            f,
            "{} at {} - {}",
            self.appointment_date_time
                .to_offset(self.local_offset.unwrap_or(UtcOffset::UTC))
                .format(&date_format)
                .unwrap(),
            self.appointment_time.format(&time_format).unwrap(),
            self.appointment_type
        )
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum FetchRange {
    ThisMonthFromToday,
    NumberOfDays(u32),
}

#[derive(Debug, Clone)]
#[repr(u32)]
#[allow(dead_code)]
pub enum Temple {
    Logan = 3,
    Oakland = 46,
}

impl Display for Temple {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Temple::Logan => "Logan Utah Temple",
                Temple::Oakland => "Oakland California Temple",
            }
        )
    }
}
