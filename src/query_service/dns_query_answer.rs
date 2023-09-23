#[derive(derive_builder::Builder, Default, Debug)]
pub struct DNSQueryAnswer<'a> {
    message_id: u16,
    op_code: u8,
    is_authoritative: bool,
    is_truncated: bool,
    is_recursion_desired: bool,
    is_recursion_available: bool,
    is_answer_authenticated: bool,
    is_non_auth_answer_acceptable: bool,
    r_code: u8,
    num_of_questions: u16,
    num_of_answers: u16,
    num_of_authorities: u16,
    num_of_additional_rrs: u16,
    q_name_array: Vec<&'a str>,
    ans_array: Vec<&'a str>,
}

impl<'a> Into<Vec<u8>> for DNSQueryAnswer<'a> {
    fn into(self) -> Vec<u8> {
        Vec::new()
    }
}
