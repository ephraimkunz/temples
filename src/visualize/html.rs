use crate::{data::Day, Temple};
use anyhow::Result;
use std::collections::HashMap;
use std::io::Write;
use time::macros::format_description;

use super::OutputWriter;

pub struct HTMLWriter;

impl OutputWriter for HTMLWriter {
    fn write_output(schedules: &[Day], temple: &Temple, filename: &str) -> Result<()> {
        let mut output = std::fs::File::create(format!("{filename}.html"))?;

        const START_HOUR: u8 = 5;
        const END_HOUR: u8 = 20;

        let prefix = format!(
            "<!DOCTYPE html>
        <html>
        
        <head>
        <link href=\"https://cdn.jsdelivr.net/npm/bootstrap@5.1.3/dist/css/bootstrap.min.css\" rel=\"stylesheet\" integrity=\"sha384-1BmE4kWBq78iYhFldvKuhfTAU6auU8tT94WrHftjDbrCEXSU1oBoqyl2QvZ6jIW3\" crossorigin=\"anonymous\">
            <style>
                .grid-container {{
                    display: grid;
                    grid-template-columns: repeat({num_columns}, 1fr);
                    grid-template-rows: repeat({num_rows}, 1fr);
                    grid-auto-flow: column;
                    width: 300%;
                    height: 50%;
                }}
    
                body {{
                    padding: 20px;
                }}
        
                .grid-item {{
                    border: 1px solid rgba(0, 0, 0, 0.8);
                    padding: 5px;
                    font-size: 15px;
                    text-align: center;
                }}
        
                .header-item {{
                    grid-row-start: 0;
                }}
        
                .item1 {{
                    background: LightSkyBlue;
                }}
            </style>
        </head>
        
        <body>
            <h1>{temple_name}</h1>
            <p>Available slots for endowment</p>
            <div class=\"grid-container\">",
            num_columns = schedules.len() + 1,
            num_rows = (END_HOUR - START_HOUR) * 2 + 1,
            temple_name = temple.name
        );

        writeln!(output, "{}", prefix)?;

        writeln!(output, "<div class=\"grid-item\"></div>")?; // Blank in corner

        for hour in START_HOUR..END_HOUR {
            for minutes in [0, 30] {
                if hour <= 12 {
                    writeln!(
                        output,
                        "<div class=\"grid-item\">{}:{:02} AM</div>",
                        hour, minutes
                    )?;
                } else {
                    writeln!(
                        output,
                        "<div class=\"grid-item\">{}:{:02} PM</div>",
                        hour - 12,
                        minutes
                    )?;
                }
            }
        }

        let date_format = format_description!("[weekday repr:short] [month repr:short] [day]");
        for day in schedules {
            writeln!(
                output,
                "<div class=\"grid-item\">{}</div>",
                day.date.format(date_format).unwrap()
            )?;

            let hour_counts: HashMap<(u8, u8), u32> = day
                .sessions
                .session_list
                .iter()
                .map(|s| {
                    (
                        (s.time.hour(), s.time.minute()),
                        if s.details.remaining_online_seats_available < 0 {
                            0
                        } else {
                            s.details.remaining_online_seats_available
                        } as u32,
                    )
                })
                .collect();
            for hour in START_HOUR..END_HOUR {
                for minutes in [0, 30] {
                    match hour_counts.get(&(hour, minutes)) {
                        Some(&remaining) => {
                            let background_color = if remaining > 0 {
                                "bg-success"
                            } else {
                                "bg-danger"
                            };
                            writeln!(output, "<div class=\"grid-item text-white {background_color}\">{remaining}</div>")?
                        }
                        None => writeln!(output, "<div class=\"grid-item bg-secondary\"></div>")?,
                    }
                }
            }
        }

        let postfix = "       </div>
        </body>
        
        </html>";
        writeln!(output, "{}", postfix)?;

        Ok(())
    }
}
