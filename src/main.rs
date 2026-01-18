#[allow(unused_imports)]
use std::net::UdpSocket;
mod dns;
use dns::Headers;

use crate::dns::{Answer, DNSPacket, Question};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // TODO: Uncomment the code below to pass the first stage
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                let packet = DNSPacket::from_bytes(&buf);
                println!("Received {} bytes from {}", size, source);
                let headers = Headers {
                    packet_id: packet.headers.packet_id,
                    query_response_indicator: true,
                    operation_code: packet.headers.operation_code,
                    authoritative_answer: false,
                    truncation: false,
                    recursion_desired: packet.headers.recursion_desired,
                    recursion_available: false,
                    reserved: 0,
                    response_code: if packet.headers.operation_code == 0 {
                        0
                    } else {
                        4
                    },
                    question_count: packet.headers.question_count,
                    answer_record_count: packet.headers.question_count,
                    authority_record_count: 0,
                    additional_record_count: 0,
                };
                let mut response: Vec<u8> = headers.into();

                for qn in &packet.questions {
                    let question = Question {
                        name: qn.name.clone(),
                        record_type: qn.record_type,
                        class: qn.class,
                    };
                    let question_bytes: Vec<u8> = question.into();
                    response.extend_from_slice(&question_bytes);
                }

                for qn in &packet.questions {
                    let answer = Answer {
                        name: qn.name.clone(),
                        record_type: qn.record_type,
                        class: qn.class,
                        ttl: 60,
                        data: vec![8, 8, 8, 8],
                    };
                    let answer_bytes: Vec<u8> = answer.into();
                    response.extend_from_slice(&answer_bytes);
                }

                udp_socket
                    .send_to(&response, source)
                    .expect("Failed to send response");
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }
}
