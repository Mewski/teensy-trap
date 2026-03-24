use crate::host::mouse::MouseReport;

pub struct Forwarder {
  report_count: u64,
  drop_count: u64,
}

impl Forwarder {
  pub const fn new() -> Self {
    Self {
      report_count: 0,
      drop_count: 0,
    }
  }

  pub fn forward(&mut self, report: &MouseReport, output: &mut [u8]) -> usize {
    self.report_count += 1;
    let _ = (report, output);
    todo!()
  }

  pub fn stats(&self) -> (u64, u64) {
    (self.report_count, self.drop_count)
  }
}
