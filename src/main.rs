#[allow(unused_imports)]
use std::net::UdpSocket;
mod dns;
use clap::Parser;
use dns::Headers;

use crate::dns::{Answer, DNSPacket, Question};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    // Resolver
    #[arg(long)]
    resolver: Option<String>,
}

fn main() {
    let args = Args::parse();
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

                match args.resolver.as_ref() {
                    Some(resolver) => {
                        println!("Forwarding packet to {}", resolver);
                        let upstream_socket =
                            UdpSocket::bind("0.0.0.0:0").expect("Failed to bind to address");

                        for i in 0..packet.headers.question_count {
                            // split questions into single packets
                            let i = i as usize;
                            let mut packet_clone = packet.clone();
                            packet_clone.headers.question_count = 1;
                            packet_clone.questions = Vec::from(&packet_clone.questions[i..i + 1]);
                            let packet_buf: Vec<u8> = packet_clone.into();
                            upstream_socket
                                .send_to(&packet_buf, resolver)
                                .expect("Failed to forward DNS packet");
                            let _ = upstream_socket
                                .recv_from(&mut buf)
                                .expect("Expected a response");
                            let forwarded_response_packet = DNSPacket::from_bytes(&buf);
                            println!("{}: {:?}", i, forwarded_response_packet);

                            for ans in forwarded_response_packet.answers {
                                let answer_bytes: Vec<u8> = ans.into();
                                response.extend_from_slice(&answer_bytes);
                            }
                        }
                    }
                    None => {
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
                    }
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
