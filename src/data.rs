use serde::{de, Deserialize, Deserializer, Serialize};
use std::{fmt, fmt::Display, str::FromStr};
use time::{
    macros::format_description, serde::rfc3339, Date, OffsetDateTime, PrimitiveDateTime, Time,
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

mod serde_date {
    use serde::{de, Deserializer, Serializer};
    use std::fmt;
    use time::macros::format_description;
    use time::Date;

    pub fn serialize<S>(date: &Option<Date>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(date) = date {
            let format = format_description!("[day padding:none] [month repr:long] [year]");
            let formatted_date = date.format(&format).unwrap();
            serializer.serialize_str(&formatted_date)
        } else {
            serializer.serialize_none()
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Date>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct JsonStringVisitor;

        impl<'de> de::Visitor<'de> for JsonStringVisitor {
            type Value = Option<Date>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string containing a time in <day month year> format")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let format = format_description!("[day padding:none] [month repr:long] [year]");
                Ok(Date::parse(v, &format).ok())
            }
        }

        deserializer.deserialize_any(JsonStringVisitor)
    }
}

mod serde_status {
    use crate::data::Status;
    use serde::{de, Deserializer, Serializer};
    use std::fmt;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Status, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct JsonStringVisitor;

        impl<'de> de::Visitor<'de> for JsonStringVisitor {
            type Value = Status;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string containing a known status")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match v.to_uppercase().as_str() {
                    "RENOVATION" => Ok(Status::Renovation),
                    "CONSTRUCTION" => Ok(Status::Construction),
                    "OPERATING" => Ok(Status::Operating),
                    "ANNOUNCED" => Ok(Status::Announced),
                    other => Err(E::custom(format!(
                        "Unable to parse status from string {}",
                        other
                    ))),
                }
            }
        }

        deserializer.deserialize_any(JsonStringVisitor)
    }

    pub fn serialize<S>(status: &Status, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(match status {
            Status::Renovation => "RENOVATION",
            Status::Construction => "CONSTRUCTION",
            Status::Operating => "OPERATING",
            Status::Announced => "ANNOUNCED",
        })
    }
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
}

pub enum OrdinanceType {
    Baptism,
    Initiatory,
    Endowment,
    Sealing,
}

impl Display for OrdinanceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OrdinanceType::Baptism => write!(f, "Baptism"),
            OrdinanceType::Initiatory => write!(f, "Initiatory"),
            OrdinanceType::Endowment => write!(f, "Endowment"),
            OrdinanceType::Sealing => write!(f, "Sealing"),
        }
    }
}

impl FromStr for OrdinanceType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "BAPTISM" => Ok(OrdinanceType::Baptism),
            "INITIATORY" => Ok(OrdinanceType::Initiatory),
            "ENDOWMENT" => Ok(OrdinanceType::Endowment),
            "SEALING" => Ok(OrdinanceType::Sealing),
            "PROXY_SEALING" => Ok(OrdinanceType::Sealing),
            o => Err(format!("Unknown ordinance type {}", o)),
        }
    }
}

impl AppointmentJSON {
    pub fn ordinance_type(&self) -> OrdinanceType {
        self.appointment_type
            .parse()
            .expect("Unable to parse appointment type")
    }
}

impl Display for AppointmentJSON {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let date_format = format_description!("[month repr:short] [day padding:none], [year]");
        let time_format = format_description!("[hour repr:12 padding:none]:[minute] [period]");

        write!(
            f,
            "{} at {} - {}",
            self.appointment_date_time.format(&date_format).unwrap(),
            self.appointment_time.format(&time_format).unwrap(),
            self.ordinance_type()
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
pub enum Status {
    Construction,
    Operating,
    Announced,
    Renovation,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(unused)]
pub struct Temple {
    pub name: String,
    #[serde(with = "serde_status")]
    pub status: Status,

    #[serde(with = "serde_date")]
    pub date: Option<Date>,

    pub temple_org_id: u32,

    pub country: String,

    location: String,
    temple_name_id: String,
    city: String,
    state_region: String,
    sort_date: String,
}
