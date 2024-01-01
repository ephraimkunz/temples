use anyhow::Result;
use time::{OffsetDateTime, PrimitiveDateTime, Time};

use crate::{
    client::Client,
    data::{AppointmentJSON, Day, FetchRange, SessionsJSON, Temple},
};

pub fn get_appointments(client: &Client) -> Result<Vec<AppointmentJSON>> {
    // Fetch appointments.
    let appointments: Vec<AppointmentJSON> =
        ureq::get("https://tos.churchofjesuschrist.org/api/appointments")
            .set("Cookie", &client.cookie)
            .call()?
            .into_json()?;

    Ok(appointments)
}

pub fn get_schedules(client: &Client, range: FetchRange, temple: &Temple) -> Result<Vec<Day>> {
    let now = OffsetDateTime::now_local().expect("Unable to get local time");

    // Fetch schedules for the rest of the month.
    let mut num_days_fetched = 0;
    let mut days = vec![];
    let mut next_date = now.date();
    'fetch_loop: loop {
        let sessions: SessionsJSON =
            ureq::post("https://tos.churchofjesuschrist.org/api/templeSchedule/getSessionInfo")
                .set("Cookie", &client.cookie)
                .send_json(ureq::json!({
                    "sessionYear":next_date.year(),
                    "sessionMonth":next_date.month() as u8 - 1,
                    "sessionDay":next_date.day(),
                    "appointmentType":"PROXY_ENDOWMENT",
                    "templeOrgId":temple.temple_org_id
                }))?
                .into_json()?;

        num_days_fetched += 1;

        days.push(Day {
            date: PrimitiveDateTime::new(next_date, Time::MIDNIGHT).assume_offset(now.offset()),
            sessions,
        });

        match next_date.next_day() {
            Some(next) => {
                match range {
                    FetchRange::NumberOfDays(n) => {
                        if num_days_fetched >= n {
                            break;
                        }
                    }
                    FetchRange::ThisMonthFromToday => {
                        if next.month() != now.month() {
                            break 'fetch_loop;
                        }
                    }
                }

                next_date = next;
            }
            None => break 'fetch_loop,
        }
    }

    Ok(days)
}

pub fn get_temples() -> Result<Vec<Temple>> {
    const START_DELIMITER: &str = "templeList\":";
    const END_DELIMITER: &str = "}]";
    let html = ureq::get("https://www.churchofjesuschrist.org/temples/list")
        .call()?
        .into_string()?;

    let json_start = html
        .find(START_DELIMITER)
        .map(|i| i + START_DELIMITER.len())
        .ok_or(anyhow::anyhow!("Couldn't find start of temple data"))?;
    let json_end = html[json_start..]
        .find(END_DELIMITER)
        .map(|i| i + json_start + END_DELIMITER.len())
        .ok_or(anyhow::anyhow!("Couldn't find end of temple data"))?;
    let json_string = &html[json_start..json_end];

    let temples: Vec<Temple> = serde_json::de::from_str(json_string)?;
    Ok(temples)
}
