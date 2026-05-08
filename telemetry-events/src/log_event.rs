use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct LogEvent {
    pub category: LogCategory,
    pub message:  heapless::String<256>,
}

impl LogEvent {
    pub fn encode<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        self.category.encode(&mut buf[0]);
        buf[1..1 + self.message.len()].copy_from_slice(self.message.as_bytes());
        return &buf[0..1 + self.message.len()];
    }

    pub fn decode(buf: &[u8]) -> Option<Self> {
        if buf.len() <= 2 {
            return None;
        }

        let Some(category) = LogCategory::decode(buf[0]) else {
            return None;
        };

        let message = core::str::from_utf8(&buf[1..]).ok()?;
        let message = heapless::String::from_str(message).ok()?;

        Some(Self { category, message })
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum LogCategory {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogCategory {
    pub fn encode<'a>(self, buf: &mut u8) {
        *buf = self as u8;
    }

    pub fn decode(byte: u8) -> Option<Self> {
        use LogCategory::*;

        match byte {
            0 => Some(Debug),
            1 => Some(Info),
            2 => Some(Warn),
            3 => Some(Error),
            _ => None,
        }
    }

    pub fn to_str(self) -> &'static str {
        use LogCategory::*;

        match self {
            Debug => "Debug",
            Info  => "Info",
            Warn  => "Warn",
            Error => "Error",
        }
    }
}