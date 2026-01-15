use bytes::Buf;
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

impl Headers {
    pub fn from_bytes(mut buf: &[u8]) -> Headers {
        let id = buf.get_u16();
        let b1 = buf.get_u8();
        let query_response_indicator = (b1 >> 7) & 1 == 1;
        let operation_code = (b1 >> 3) & 0b1111;
        let authoritative_answer = (b1 >> 2) & 1 == 1;
        let truncation = (b1 >> 1) & 1 == 1;
        let recursion_desired = b1 & 1 == 1;

        let b2 = buf.get_u8();
        let recursion_available = (b2 >> 7) & 1 == 1;
        let reserved = (b2 >> 4) & 0b111;
        let response_code = b2 & 0b1111;
        let question_count = buf.get_u16();
        let answer_record_count = buf.get_u16();
        let authority_record_count = buf.get_u16();
        let additional_record_count = buf.get_u16();

        Headers {
            packet_id: id,
            query_response_indicator,
            operation_code,
            authoritative_answer,
            truncation,
            recursion_desired,
            recursion_available,
            reserved,
            response_code,
            question_count,
            answer_record_count,
            authority_record_count,
            additional_record_count,
        }
    }
}

impl From<Headers> for Vec<u8> {
    fn from(headers: Headers) -> Self {
        let mut bytes = Vec::with_capacity(12);
        bytes.extend_from_slice(&headers.packet_id.to_be_bytes());

        // first byte
        // bit 1
        let mut b1 = (headers.query_response_indicator as u8) << 7;
        // bit 2-5
        b1 |= headers.operation_code << 3;
        // bit 6
        b1 |= (headers.authoritative_answer as u8) << 2;
        // bit 7
        b1 |= (headers.truncation as u8) << 1;
        // bit 8
        b1 |= headers.recursion_desired as u8;
        bytes.push(b1);

        // second byte
        // bit 1
        let mut b2 = (headers.recursion_available as u8) << 7;
        // bits 2-4
        b2 |= headers.reserved << 4;
        // bits 5-8
        b2 |= headers.response_code;
        bytes.push(b2);

        bytes.extend_from_slice(&headers.question_count.to_be_bytes());
        bytes.extend_from_slice(&headers.answer_record_count.to_be_bytes());
        bytes.extend_from_slice(&headers.authority_record_count.to_be_bytes());
        bytes.extend_from_slice(&headers.additional_record_count.to_be_bytes());

        bytes
    }
}

pub struct Question {
    pub name: Vec<String>,
    pub record_type: u16,
    pub class: u16,
}

impl Question {
    fn from_bytes(mut buf: &[u8]) -> Self {
        let mut len = buf.get_u8();
        let mut name = Vec::<String>::new();
        while len > 0 {
            println!("str len: {len}");
            let str_bytes = buf.copy_to_bytes(len as usize);
            let s = str::from_utf8(&str_bytes).expect("invalid bytes");
            println!("read str: {s}");
            name.push(s.into());
            len = buf.get_u8();
        }

        let record_type = buf.get_u16();
        let class = buf.get_u16();
        Question {
            name,
            record_type,
            class,
        }
    }
}

impl From<Question> for Vec<u8> {
    fn from(question: Question) -> Self {
        let mut bytes = Vec::new();

        for label in question.name {
            let size = label.len() as u8;
            bytes.extend_from_slice(&size.to_be_bytes());
            bytes.extend_from_slice(label.as_bytes());
        }

        bytes.push(b'\0');

        bytes.extend_from_slice(&question.record_type.to_be_bytes());
        bytes.extend_from_slice(&question.class.to_be_bytes());

        bytes
    }
}

pub struct Answer {
    pub name: Vec<String>,
    pub record_type: u16,
    pub class: u16,
    pub ttl: u32,
    pub data: Vec<u8>,
}

impl From<Answer> for Vec<u8> {
    fn from(answer: Answer) -> Self {
        let mut bytes = Vec::new();

        for label in answer.name {
            let size = label.len() as u8;
            bytes.extend_from_slice(&size.to_be_bytes());
            bytes.extend_from_slice(label.as_bytes());
        }
        bytes.push(b'\0');

        bytes.extend_from_slice(&answer.record_type.to_be_bytes());
        bytes.extend_from_slice(&answer.class.to_be_bytes());
        bytes.extend_from_slice(&answer.ttl.to_be_bytes());

        bytes.extend_from_slice(&answer.data.len().to_be_bytes());
        bytes.extend_from_slice(&answer.data);

        bytes
    }
}

pub struct DNSPacket {
    pub headers: Headers,
    pub question: Question,
}

impl DNSPacket {
    pub fn from_bytes(buf: &[u8]) -> Self {
        let headers = Headers::from_bytes(&buf[0..12]);
        let question = Question::from_bytes(&buf[12..]);
        DNSPacket { headers, question }
    }
}
