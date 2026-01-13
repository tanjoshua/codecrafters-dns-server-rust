pub struct Headers {
    pub packet_id: u16,
    pub query_response_indicator: bool,
    pub operation_code: u8, // 4bits
    pub authoritative_answer: bool,
    pub truncation: bool,
    pub recursion_desired: bool,
    pub recursion_available: bool,
    pub reserved: u8,      // 3 bits
    pub response_code: u8, // 4 bits
    pub question_count: u16,
    pub answer_record_count: u16,
    pub authority_record_count: u16,
    pub additional_record_count: u16,
}

impl Into<Vec<u8>> for Headers {
    fn into(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(12);
        bytes.extend_from_slice(&self.packet_id.to_be_bytes());

        // first byte
        // bit 1
        let mut b1 = (self.query_response_indicator as u8) << 7;
        // bit 2-5
        b1 |= self.operation_code << 3;
        // bit 6
        b1 |= (self.authoritative_answer as u8) << 2;
        // bit 7
        b1 |= (self.truncation as u8) << 1;
        // bit 8
        b1 |= self.recursion_desired as u8;
        bytes.push(b1);

        // second byte
        // bit 1
        let mut b2 = (self.recursion_available as u8) << 7;
        // bits 2-4
        b2 |= self.reserved << 4;
        // bits 5-8
        b2 |= self.response_code;
        bytes.push(b2);

        bytes.extend_from_slice(&self.question_count.to_be_bytes());
        bytes.extend_from_slice(&self.answer_record_count.to_be_bytes());
        bytes.extend_from_slice(&self.authority_record_count.to_be_bytes());
        bytes.extend_from_slice(&self.additional_record_count.to_be_bytes());

        bytes
    }
}
