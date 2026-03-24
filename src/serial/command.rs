use crate::proxy::inject::InjectionCommand;

const HEADER: u8 = 0xAA;
const FRAME_LEN: usize = 6;

enum ParseState {
  WaitHeader,
  ReadPayload { idx: usize },
}

pub struct CommandParser {
  state: ParseState,
  buf: [u8; FRAME_LEN],
}

impl CommandParser {
  pub const fn new() -> Self {
    Self {
      state: ParseState::WaitHeader,
      buf: [0; FRAME_LEN],
    }
  }

  pub fn feed(&mut self, byte: u8) -> Option<InjectionCommand> {
    match &mut self.state {
      ParseState::WaitHeader => {
        if byte == HEADER {
          self.state = ParseState::ReadPayload { idx: 0 };
        }
        None
      }
      ParseState::ReadPayload { idx } => {
        self.buf[*idx] = byte;
        *idx += 1;
        if *idx >= FRAME_LEN {
          let cmd = self.parse();
          self.state = ParseState::WaitHeader;
          cmd
        } else {
          None
        }
      }
    }
  }

  fn parse(&self) -> Option<InjectionCommand> {
    let dx = i16::from_le_bytes([self.buf[0], self.buf[1]]);
    let dy = i16::from_le_bytes([self.buf[2], self.buf[3]]);

    Some(InjectionCommand {
      dx,
      dy,
      buttons_override: if self.buf[4] != 0xFF { Some(self.buf[4]) } else { None },
      wheel_override: if self.buf[5] != 0x80 {
        Some(self.buf[5] as i8)
      } else {
        None
      },
    })
  }
}
