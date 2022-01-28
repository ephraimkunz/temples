use crate::data::{Day, Temple};
use anyhow::Result;

use self::{excel::ExcelWriter, html::HTMLWriter};

mod excel;
mod html;

#[allow(dead_code)]
pub enum OutputFormat {
    Html,
    Excel,
}

trait OutputWriter {
    fn write_output(schedules: &[Day], temple: &Temple) -> Result<()>;
}

pub fn write_output(schedules: &[Day], temple: &Temple, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Html => HTMLWriter::write_output(schedules, temple),
        OutputFormat::Excel => ExcelWriter::write_output(schedules, temple),
    }
}
