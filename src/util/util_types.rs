#![allow(dead_code)]

#[derive(derive_builder::Builder, Default, Debug)]
pub struct DNSQuery<'a> {
    op_code: u8,
    is_truncated: bool,
    is_recursive: bool,
    num_of_questions: u16,
    num_of_arr: u16,
    num_of_ar: u16,
    num_of_additional_rrs: u16,
    q_name_array: Vec<&'a str>,
    q_type: QType,
    q_class: QClass,
}

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
