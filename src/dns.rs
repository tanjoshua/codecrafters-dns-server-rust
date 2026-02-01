use bytes::Buf;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct Question {
    pub name: Vec<String>,
    pub record_type: u16,
    pub class: u16,
}

pub fn get_name(buf: &[u8], pos: usize) -> (Vec<String>, usize) {
    let mut pos = pos;
    let mut name = Vec::<String>::new();

    loop {
        // 1. check if pointer or label
        let b1 = buf[pos];
        if (b1 >> 6) == 0b11 {
            let val = u16::from_be_bytes([buf[pos], buf[pos + 1]]);
            let offset = ((val << 2) >> 2) as usize;
            let (res, _) = get_name(buf, offset);
            name.extend(res);
            pos += 2; // pointer is 2 bytes
            break;
        }

        // label
        let len = buf[pos] as usize;
        pos += 1;

        // break if null byte
        if len == 0 {
            break;
        }

        // get value
        let s = str::from_utf8(&buf[pos..(pos + len)]).expect("bytes have to be valid string");
        pos += len;
        name.push(s.into());
    }

    (name, pos)
}

impl Question {
    fn from_bytes(buf: &[u8], pos: usize) -> (Self, usize) {
        let mut pos = pos;
        let (name, new_pos) = get_name(buf, pos);
        pos = new_pos;

        let record_type = u16::from_be_bytes([buf[pos], buf[pos + 1]]);
        let class = u16::from_be_bytes([buf[pos + 2], buf[pos + 3]]);
        pos += 4;
        (
            Question {
                name,
                record_type,
                class,
            },
            pos,
        )
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

#[derive(Debug, Clone)]
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

        let rdlength = answer.data.len() as u16;
        bytes.extend_from_slice(&rdlength.to_be_bytes());
        bytes.extend_from_slice(&answer.data);

        bytes
    }
}

impl Answer {
    fn from_bytes(buf: &[u8], pos: usize) -> (Self, usize) {
        let mut pos = pos;
        let (name, new_pos) = get_name(buf, pos);
        pos = new_pos;

        let record_type = u16::from_be_bytes([buf[pos], buf[pos + 1]]);
        let class = u16::from_be_bytes([buf[pos + 2], buf[pos + 3]]);
        let ttl = u32::from_be_bytes([buf[pos + 4], buf[pos + 5], buf[pos + 6], buf[pos + 7]]);
        pos += 8;

        let data_len = u16::from_be_bytes([buf[pos], buf[pos + 1]]) as usize;
        pos += 2;

        let data = Vec::from(&buf[pos..pos + data_len]);
        pos += data_len;

        (
            Answer {
                name,
                record_type,
                class,
                ttl,
                data,
            },
            pos,
        )
    }
}

#[derive(Debug, Clone)]
pub struct DNSPacket {
    pub headers: Headers,
    pub questions: Vec<Question>,
    pub answers: Vec<Answer>,
}

impl DNSPacket {
    pub fn from_bytes(buf: &[u8]) -> Self {
        let headers = Headers::from_bytes(buf);
        let mut questions = Vec::new();
        let mut pos = 12;
        for _ in 0..headers.question_count {
            let (question, new_pos) = Question::from_bytes(buf, pos);
            questions.push(question);
            pos = new_pos;
        }

        let mut answers = Vec::new();
        for _ in 0..headers.answer_record_count {
            let (answer, new_pos) = Answer::from_bytes(buf, pos);
            answers.push(answer);
            pos = new_pos;
        }
        DNSPacket {
            headers,
            questions,
            answers,
        }
    }
}

impl From<DNSPacket> for Vec<u8> {
    fn from(packet: DNSPacket) -> Self {
        let mut bytes: Vec<u8> = packet.headers.into();

        for qn in packet.questions {
            let question_bytes: Vec<u8> = qn.into();
            bytes.extend_from_slice(&question_bytes);
        }

        for ans in packet.answers {
            let ans_bytes: Vec<u8> = ans.into();
            bytes.extend_from_slice(&ans_bytes);
        }

        bytes
    }
}
