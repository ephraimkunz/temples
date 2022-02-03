use crate::data::{Day, Temple};
use anyhow::Result;
use clap::ArgEnum;

use self::{excel::ExcelWriter, html::HTMLWriter};

mod excel;
mod html;

#[allow(dead_code)]
#[derive(ArgEnum, Clone)]
pub enum ScheduleOutputFormat {
    Html,
    Excel,
}

trait OutputWriter {
    fn write_output(schedules: &[Day], temple: &Temple, filename: &str) -> Result<()>;
}

pub fn write_output(
    schedules: &[Day],
    temple: &Temple,
    format: ScheduleOutputFormat,
    filename: &str,
) -> Result<()> {
    match format {
        ScheduleOutputFormat::Html => HTMLWriter::write_output(schedules, temple, filename),
        ScheduleOutputFormat::Excel => ExcelWriter::write_output(schedules, temple, filename),
    }
}
