use super::util_types::*;
use std::str::from_utf8;

pub fn get_query_from_bytes(
    bytes: &[u8],
) -> Result<DNSQuery, Box<dyn std::error::Error + Send + Sync>> {
    // first two bytes should be id (u16)
    // big endian
    let index: u16 = (bytes[0] as u16) << 8 | bytes[1] as u16;
    println!("index: {}", index);

    // second two bytes should be flags
    // going to skip the second byte for now
    let _is_question = bytes[2] & 0b1000_0000 == 0;
    let op_code = bytes[2] & 0b0111_1000 >> 3;
    let is_truncated = bytes[2] & 0b0000_0010 != 0;
    let is_recursive = bytes[2] & 0b0000_0001 != 0;

    // starting from 4th index
    let num_of_questions: u16 = (bytes[4] as u16) << 8 | bytes[5] as u16;
    let num_of_arr: u16 = (bytes[6] as u16) << 8 | bytes[7] as u16;
    let num_of_ar: u16 = (bytes[8] as u16) << 8 | bytes[9] as u16;
    let num_of_additional_rrs: u16 = (bytes[10] as u16) << 8 | bytes[11] as u16;

    // answer section starting from 12th index
    let mut cur_idx = 12;
    let mut q_name_array: Vec<&str> = Vec::new();
    while bytes[cur_idx] != 0x00 {
        let len = bytes[cur_idx];
        let segment = from_utf8(&bytes[cur_idx + 1..cur_idx + 1 + len as usize])?;
        q_name_array.push(segment);
        cur_idx += len as usize + 1;
    }
    cur_idx += 1;

    // query type
    let q_type = (bytes[cur_idx] as u16) << 8 | bytes[cur_idx + 1] as u16;
    cur_idx += 2;

    // q class
    let q_class = (bytes[cur_idx] as u16) << 8 | bytes[cur_idx + 1] as u16;
    cur_idx += 2;

    Ok(DNSQueryBuilder::default()
        .op_code(op_code)
        .is_truncated(is_truncated)
        .is_recursive(is_recursive)
        .num_of_questions(num_of_questions)
        .num_of_arr(num_of_arr)
        .num_of_ar(num_of_ar)
        .num_of_additional_rrs(num_of_additional_rrs)
        .q_name_array(q_name_array)
        .q_type(QType::from_u16(q_type).ok_or("QType::from_u16 failed")?)
        .q_class(QClass::from_u16(q_class).ok_or("QClass::from_u16 failed")?)
        .build()?)
}
