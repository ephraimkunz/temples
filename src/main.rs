use std::collections::HashMap;

use crate::network::get_temples;
use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use client::Client;
use data::{FetchRange, Temple};
use network::{get_appointments, get_schedules};
use term_table::row::Row;
use term_table::table_cell::TableCell;
use time::macros::format_description;
use visualize::ScheduleOutputFormat;

mod client;
mod data;
mod network;
mod visualize;

#[derive(Parser)]
#[clap(version)]
#[clap(author = "Ephraim Kunz <ephraimkunz@me.com>")]
#[clap(about = "Do fun stuff with temple data.", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get list of temples
    Temples {
        /// Which format to use for outputting the list of temples
        #[clap(short, long, value_enum, default_value_t = TempleOutputFormat::Table)]
        format: TempleOutputFormat,
    },

    /// Get existing temple appointments
    Appointments {},

    /// Get a temple's endowment schedule
    Schedules {
        /// Temple id
        #[clap(short, long)]
        id: u32,

        /// How many days to fetch. Use 0 to fetch from now until the end of the month.
        #[clap(short, long, default_value_t = 0)]
        count: u32,

        /// Format that schedule is output in
        #[clap(short = 'o', long = "output", value_enum, default_value_t = ScheduleOutputFormat::Excel)]
        format: ScheduleOutputFormat,

        /// The name of the output file
        #[clap(short, long, default_value_t = String::from("schedule"))]
        filename: String,
    },
}

#[derive(ValueEnum, Clone)]
enum TempleOutputFormat {
    /// ASCII table
    Table,

    /// Full information in JSON
    Json,

    /// Histogram of year of temple dedication
    HistogramYear,

    /// Histogram of temple country
    HistogramCountry,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Temples { format } => {
            let temples = get_temples()?;
            match format {
                TempleOutputFormat::Table => {
                    let mut table = term_table::Table::new();
                    table.max_column_width = 80;
                    table.style = term_table::TableStyle::extended();

                    table.add_row(Row::new([
                        TableCell::new("Name"),
                        TableCell::new("Status"),
                        TableCell::new("Dedicated"),
                        TableCell::new("Id"),
                    ]));

                    let date_format =
                        format_description!("[month repr:short] [day padding:none], [year]");

                    let count = temples.len();
                    for temple in temples {
                        table.add_row(Row::new([
                            TableCell::new(temple.name),
                            TableCell::new(format!("{:?}", temple.status)),
                            TableCell::new(
                                temple
                                    .date
                                    .and_then(|d| d.format(&date_format).ok())
                                    .unwrap_or_else(|| "".to_string()),
                            ),
                            TableCell::new(temple.temple_org_id),
                        ]));
                    }

                    table.add_row(Row::new([TableCell::new(format!(
                        "Total Count: {}",
                        count
                    ))]));

                    println!("{}", table.render());
                }
                TempleOutputFormat::Json => {
                    println!("{}", serde_json::ser::to_string_pretty(&temples)?)
                }
                TempleOutputFormat::HistogramYear => {
                    let mut histogram = HashMap::new();
                    for temple in temples {
                        if let Some(date) = temple.date {
                            let count = histogram.entry(date.year()).or_insert(0);
                            *count += 1;
                        }
                    }

                    print_histogram(&histogram);
                }

                TempleOutputFormat::HistogramCountry => {
                    let mut histogram = HashMap::new();
                    for temple in temples {
                        if !temple.country.is_empty() {
                            let count = histogram.entry(temple.country).or_insert(0);
                            *count += 1;
                        }
                    }

                    print_histogram(&histogram);
                }
            }
        }
        Commands::Appointments {} => {
            let client = Client::new()?;

            let appointments = get_appointments(&client)?;
            for appointment in appointments {
                println!("{appointment}");
            }
        }
        Commands::Schedules {
            id,
            count,
            format,
            filename,
        } => {
            let range = if count == 0 {
                FetchRange::ThisMonthFromToday
            } else {
                FetchRange::NumberOfDays(count)
            };

            let client = Client::new()?;
            let temples = get_temples()?;
            let temple = temples
                .into_iter()
                .find(|t| t.temple_org_id == id)
                .ok_or_else(|| anyhow::anyhow!("Invalid temple id: {}", id))?;

            let schedules = get_schedules(&client, range, &temple)?;

            visualize::write_output(&schedules, &temple, format, &filename)?;
        }
    }

    Ok(())
}

fn print_histogram<K>(histogram: &HashMap<K, u32>)
where
    K: std::cmp::Ord + std::fmt::Display + std::hash::Hash,
{
    let mut keys: Vec<_> = histogram.keys().collect();
    keys.sort();

    let widths = keys.iter().map(|k| k.to_string().len());
    let max_width = widths.max().unwrap_or(0).min(20);

    for key in keys {
        print!("{:width$.prec$} ", key, width = max_width, prec = max_width);
        for _ in 0..histogram[key] {
            print!("*");
        }
        println!();
    }
}
