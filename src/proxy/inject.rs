use crate::host::mouse::MouseReport;

#[derive(Debug, Clone, Copy, defmt::Format)]
pub struct InjectionCommand {
  pub dx: i16,
  pub dy: i16,
  pub buttons_override: Option<u8>,
  pub wheel_override: Option<i8>,
}

pub struct Injector {
  pending: Option<InjectionCommand>,
}

impl Injector {
  pub const fn new() -> Self {
    Self { pending: None }
  }

  pub fn enqueue(&mut self, cmd: InjectionCommand) {
    self.pending = Some(cmd);
  }

  pub fn apply(&mut self, report: &mut MouseReport) {
    if let Some(cmd) = self.pending.take() {
      report.x = report.x.saturating_add(cmd.dx);
      report.y = report.y.saturating_add(cmd.dy);
      if let Some(buttons) = cmd.buttons_override {
        report.buttons = buttons;
      }
      if let Some(wheel) = cmd.wheel_override {
        report.wheel = wheel;
      }
    }
  }

  pub fn has_pending(&self) -> bool {
    self.pending.is_some()
  }
}
