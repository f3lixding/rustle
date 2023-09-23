#[derive(Clone, Default, Debug)]
pub enum QClass {
    #[default]
    IN,
    CS,
    CH,
    HS,
}

impl QClass {
    pub fn from_u16(u: u16) -> Option<Self> {
        match u {
            1 => Some(QClass::IN),
            2 => Some(QClass::CS),
            3 => Some(QClass::CH),
            4 => Some(QClass::HS),
            _ => None,
        }
    }
}
