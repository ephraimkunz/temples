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
            .set("Cookie", client.headers.get("Cookie").unwrap())
            .set("X-XSRF-TOKEN", client.headers.get("X-XSRF-TOKEN").unwrap())
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
                .set("Cookie", client.headers.get("Cookie").unwrap())
                .set("X-XSRF-TOKEN", client.headers.get("X-XSRF-TOKEN").unwrap())
                .send_json(ureq::json!({
                    "sessionYear":next_date.year(),
                    "sessionMonth":next_date.month() as u8 - 1,
                    "sessionDay":next_date.day(),
                    "appointmentType":"PROXY_ENDOWMENT",
                    "templeOrgId":temple.clone() as u32
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
