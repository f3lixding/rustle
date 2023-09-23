#[derive(Clone, Default, Debug)]
pub enum QType {
    #[default]
    A,
    NS,
    CNAME,
    SOA,
    PTR,
    MX,
    TXT,
    AAAA,
    SRV,
    DNAME,
    OPT,
    DS,
    RRSIG,
    DNSKEY,
    SSHFP,
    SPF,
    CAA,
}

impl QType {
    pub fn from_u16(u: u16) -> Option<Self> {
        match u {
            1 => Some(QType::A),
            2 => Some(QType::NS),
            5 => Some(QType::CNAME),
            6 => Some(QType::SOA),
            12 => Some(QType::PTR),
            15 => Some(QType::MX),
            16 => Some(QType::TXT),
            28 => Some(QType::AAAA),
            33 => Some(QType::SRV),
            39 => Some(QType::DNAME),
            41 => Some(QType::OPT),
            43 => Some(QType::DS),
            46 => Some(QType::RRSIG),
            48 => Some(QType::DNSKEY),
            53 => Some(QType::SSHFP),
            99 => Some(QType::SPF),
            257 => Some(QType::CAA),
            _ => None,
        }
    }
}
