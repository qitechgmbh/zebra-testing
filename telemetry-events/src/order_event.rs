#[derive(Debug, Clone, PartialEq)]
pub enum OrderEvent {
    Started   { order_id: u32, worker_id: Option<u32>, bounds: Option<WeightBounds> },
    Aborted   { order_id: u32 },
    Completed { order_id: u32, quantity_good: u32, quantity_scrap: u32 },
}

impl OrderEvent {
    pub fn encode<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        match self {
            OrderEvent::Started { order_id, worker_id, bounds } => {
                let mut flags: u8 = 0;
                if worker_id.is_none() { flags |= 1 << 0; }
                if bounds.is_none()    { flags |= 1 << 1; }

                buf[0] = 0;
                buf[1] = flags;

                let mut i = 2;

                buf[i..i + 4].copy_from_slice(&order_id.to_le_bytes());
                i += 4;

                if let Some(v) = worker_id {
                    buf[i..i + 4].copy_from_slice(&v.to_le_bytes());
                    i += 4;
                }

                if let Some(v) = bounds {
                    for val in [v.min, v.max, v.desired, v.trigger] {
                        buf[i..i + 2].copy_from_slice(&val.to_le_bytes());
                        i += 2;
                    }
                }

                &buf[..i]
            }

            OrderEvent::Aborted { order_id } => {
                buf[0] = 1;
                buf[1..5].copy_from_slice(&order_id.to_le_bytes());
                &buf[..5]
            }

            OrderEvent::Completed { order_id, quantity_good, quantity_scrap } => {
                buf[0] = 2;
                buf[1..5].copy_from_slice(&order_id.to_le_bytes());
                buf[5..9].copy_from_slice(&quantity_good.to_le_bytes());
                buf[9..13].copy_from_slice(&quantity_scrap.to_le_bytes());
                &buf[..13]
            }
        }
    }

    pub fn decode(buf: &[u8]) -> Option<Self> {
        if buf.len() < 1 {
            return None;
        }

        match buf[0] {
            0 => {
                if buf.len() < 6 { return None; }

                let flags = buf[1];
                let mut i = 2;

                let order_id = u32::from_le_bytes(buf[i..i+4].try_into().ok()?);
                i += 4;

                let worker_id = if flags & (1 << 0) == 0 {
                    if buf.len() < i + 4 { return None; }
                    let v = u32::from_le_bytes(buf[i..i+4].try_into().ok()?);
                    i += 4;
                    Some(v)
                } else { None };

                let bounds = if flags & (1 << 1) == 0 {
                    if buf.len() < i + 8 { return None; }

                    let min = i16::from_le_bytes(buf[i..i + 2].try_into().ok()?);
                    i += 2;

                    let max = i16::from_le_bytes(buf[i..i + 2].try_into().ok()?);
                    i += 2;

                    let desired = i16::from_le_bytes(buf[i..i + 2].try_into().ok()?);
                    i += 2;

                    let trigger = i16::from_le_bytes(buf[i..i + 2].try_into().ok()?);
                    // i += 2;

                    Some(WeightBounds {
                        min,
                        max,
                        desired,
                        trigger,
                    })
                } else { None };

                Some(OrderEvent::Started { order_id, worker_id, bounds })
            }

            1 => {
                if buf.len() < 5 { return None; }
                let order_id = u32::from_le_bytes(buf[1..5].try_into().ok()?);
                Some(OrderEvent::Aborted { order_id })
            }

            2 => {
                if buf.len() < 13 { return None; }

                let order_id       = u32::from_le_bytes(buf[1..5].try_into().ok()?);
                let quantity_good  = u32::from_le_bytes(buf[5..9].try_into().ok()?);
                let quantity_scrap = u32::from_le_bytes(buf[9..13].try_into().ok()?);

                Some(OrderEvent::Completed { order_id, quantity_good, quantity_scrap })
            }

            _ => None
        }
    }

    pub fn tag_as_str(&self)-> &'static str  {
        match self {
            Self::Started   { .. } => "Started",
            Self::Aborted   { .. } => "Aborted",
            Self::Completed { .. } => "Completed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeightBounds {
    pub min:     i16,
    pub max:     i16,
    pub desired: i16,
    pub trigger: i16,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let events = vec![
            OrderEvent::Started {
                order_id: 1,
                worker_id: Some(42),
                bounds: Some(WeightBounds { min: 1, max: 2, desired: 3, trigger: 4 }),
            },
            OrderEvent::Started {
                order_id: 2,
                worker_id: None,
                bounds: None,
            },
            OrderEvent::Aborted { order_id: 3 },
            OrderEvent::Completed {
                order_id: 4,
                quantity_good: 10,
                quantity_scrap: 2,
            },
        ];

        for event in events {
            let mut buf = [0u8; 64];
            let encoded = event.encode(&mut buf);
            let decoded = OrderEvent::decode(encoded).unwrap();
            assert_eq!(event, decoded);
        }
    }
}