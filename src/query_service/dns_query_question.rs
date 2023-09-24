use std::str::from_utf8;

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

impl<'a> TryFrom<&'a Vec<u8>> for DNSQueryQuestion<'a> {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    /// This function converts a byte array to a DNSQueryQuestion.
    /// The format of the expected byte array is as follows:
    /// - `Transaction id`: u16
    /// - `Flags`: 2 bytes are dedicated for flags:
    ///     - `is_question`:         x... .... .... ...., 1 bit
    ///     - `op_code`:             .xxx x... .... ...., 4 bits
    ///     - `is_truncated`:        .... .x.. .... ...., 1 bit
    ///     - `recurison_desired`:   .... ...x .... ...., 1 bit
    ///     - `Z reserved`:          .... .... .x.. ...., 1 bit
    ///     - `AD bit`:              .... .... ..x. ...., 1 bit
    ///     - `allow non auth data`: .... .... ...x ...., 1 bit
    /// - `Question count`: u16
    /// - `Number of Answer Resource Record`: u16.
    ///     - This is where the answer to the query question goes.
    ///     - This is not a part of the question. This is where the answer goes.
    /// - `Number of Authority Resource Records`: u16.
    ///     - This is the number of authority resource records that are capable of providing definitive
    ///     answers to the query questions.
    ///     - This is also not a part of the question.
    /// - `Number of Additional Resource Records`: u16.
    ///     - This is the number of additional resource records associated with the question.
    ///     - This is typically things that supplement the question (like querying for MX record, A
    ///     record etc).
    ///     - Some examples are:
    ///         - A or AAAA records.
    ///         - OPT records.
    ///         - TSIG records.
    /// - `Query`: variable length
    ///     - This section contains the question to be answered.
    ///     - A variable-length field that contains the domain being queried.
    ///     - It's encoded as a series of labels, each with a length byte followed by the label itself.
    ///     - Each label is a segment in the domain being queried about, without the dots (the dots is
    ///     what delimits the question, like www.google.com).
    /// - `Addition records`: Variable length. It has two possible record type. Both of which uses the
    /// same format but depending on the record type the fields are repurposed.
    ///     - Normal case. The fields in this is very similar to the query section:
    ///         - `Domain Name`: Variable length. This shares the same format as the query section.
    ///         - `Type`: u16. This correlates to `QType`.
    ///         - 'Class': u16. This correlates to `QClass`.
    ///         - `TTL`: u32, in seconds.
    ///         - `RDLENGTH`: u16, specifying length of RDATA in bytes.
    ///         - `RDATA`: The data type varies depending on the record type. For example, for an A record, it's a 32-bit IPv4 address. For an AAAA record, it's a 128-bit IPv6 address. For an OPT record, it's a series of additional fields and options.
    ///     - Special case. For OPT record the fields are repurposed.
    ///         - `Domain Name`: 0,
    ///         - `Type`: This would have the value of 41.
    ///         - `Class`: This would now be used to represent the UDP packet's size.
    ///         - `TTL`: Divided into several sub-fields including extended RCODE and flags.
    ///         - `RDLENGTH`: Length of all the options.
    ///         - `RDATA`: Variable-length field containing one or more EDNS options.
    fn try_from(bytes: &'a Vec<u8>) -> Result<Self, Self::Error> {
        if bytes.len() < 12 {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "bytes length is less than 12",
            )));
        }

        // first two bytes should be id (u16)
        // big endian
        let message_id: u16 = (bytes[0] as u16) << 8 | bytes[1] as u16;

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

        // Additional records.
        // We are reading the first byte first to determine the record type.
        let is_opt = {
            let first_byte = bytes[cur_idx] as u8;
            first_byte & 0b1111_1111 == 0b0000_0000
        };

        // Currently we are not doing anything with additional records.
        // This is because all we want to see is if the record being requested is on the blocklist.
        if is_opt {
            println!("Additional record is OPT"); // TODO: Change this to logging later
            cur_idx += 1;
            let _q_type = {
                let q_type = (bytes[cur_idx] as u16) << 8 | bytes[cur_idx + 1] as u16;
                cur_idx += 2;
                QType::from_u16(q_type).ok_or(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "QType::from_u16 failed",
                ))?
            };
            let _packet_size = {
                let packet_size = (bytes[cur_idx] as u16) << 8 | bytes[cur_idx + 1] as u16;
                cur_idx += 2;
                packet_size
            };

            let _r_code = (bytes[cur_idx] & 0b0000_1111) as u8;
        } else {
            println!("Additional records is not OPT"); // TODO: Change this to logging later
        }

        Ok(DNSQueryQuestionBuilder::default()
            .message_id(message_id)
            .op_code(op_code)
            .is_truncated(is_truncated)
            .is_recursive(is_recursive)
            .num_of_questions(num_of_questions)
            .num_of_arr(num_of_arr)
            .num_of_ar(num_of_ar)
            .num_of_additional_rrs(num_of_additional_rrs)
            .q_name_array(q_name_array)
            .q_type(QType::from_u16(q_type).ok_or(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "QType::from_u16 failed",
            ))?)
            .q_class(QClass::from_u16(q_class).ok_or(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "QClass::from_u16 failed",
            ))?)
            .build()?)
    }
}
