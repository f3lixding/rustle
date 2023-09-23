use super::{q_class::QClass, q_type::QType};

#[derive(derive_builder::Builder, Default, Debug)]
#[allow(dead_code)]
pub struct DNSQueryQuestion<'a> {
    message_id: u16,
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
