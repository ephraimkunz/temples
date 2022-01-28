use anyhow::Result;
use client::Client;
use data::{FetchRange, Temple};
use network::{get_appointments, get_schedules};

mod client;
mod data;
mod html;
mod network;

fn main() -> Result<()> {
    env_logger::init();

    let client = Client::new()?;
    let temple = Temple::Logan;
    let range = FetchRange::NumberOfDays(45);

    let appointments = get_appointments(&client)?;
    println!("Existing appointments:");
    for appointment in appointments {
        println!("{appointment}");
    }

    println!();

    let schedules = get_schedules(&client, range, &temple)?;
    println!("Sessions with available seats:");
    for schedule in &schedules {
        println!("{schedule}");
    }

    html::write_html_grid(&schedules, &temple)?;

    Ok(())
}