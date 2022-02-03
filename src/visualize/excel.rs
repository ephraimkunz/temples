use crate::{data::Day, Temple};
use anyhow::Result;
use std::collections::HashMap;
use time::macros::format_description;
use xlsxwriter::{FormatAlignment, FormatColor, Workbook};

use super::OutputWriter;

pub struct ExcelWriter;

impl OutputWriter for ExcelWriter {
    fn write_output(schedules: &[Day], temple: &Temple, filename: &str) -> Result<()> {
        const START_HOUR: u8 = 5;
        const END_HOUR: u8 = 20;

        let workbook = Workbook::new(&format!("{filename}.xlsx"));
        let title_format = workbook.add_format().set_bold();
        let subtitle_format = workbook.add_format();
        let red_format = workbook
            .add_format()
            .set_bg_color(FormatColor::Red)
            .set_align(FormatAlignment::Center);
        let green_format = workbook
            .add_format()
            .set_bg_color(FormatColor::Green)
            .set_align(FormatAlignment::Center);
        let blank_format = workbook
            .add_format()
            .set_bg_color(FormatColor::Gray)
            .set_align(FormatAlignment::Center);

        let mut sheet = workbook.add_worksheet(None)?;

        sheet.merge_range(
            0,
            0,
            0,
            (schedules.len() + 1) as u16,
            temple.name.as_str(),
            Some(&title_format),
        )?;
        sheet.merge_range(
            1,
            0,
            1,
            (schedules.len() + 1) as u16,
            "Available slots for endowment",
            Some(&subtitle_format),
        )?;

        let mut row = 3;
        for hour in START_HOUR..END_HOUR {
            for minutes in [0, 30] {
                if hour <= 12 {
                    sheet.write_string(row, 0, &format!("{}:{:02} AM", hour, minutes), None)?;
                } else {
                    sheet.write_string(
                        row,
                        0,
                        &format!("{}:{:02} PM", hour - 12, minutes),
                        None,
                    )?;
                }

                row += 1;
            }
        }

        let date_format = format_description!("[weekday repr:short] [month repr:short] [day]");
        let mut col = 1;
        for day in schedules {
            let mut row = 2;

            sheet.write_string(row, col, &day.date.format(&date_format).unwrap(), None)?;

            row += 1;

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
                            let format = if remaining > 0 {
                                &green_format
                            } else {
                                &red_format
                            };
                            sheet.write_number(row, col, remaining.into(), Some(format))?;
                        }
                        None => sheet.write_blank(row, col, Some(&blank_format))?,
                    }

                    row += 1;
                }
            }

            col += 1;
        }

        Ok(())
    }
}
