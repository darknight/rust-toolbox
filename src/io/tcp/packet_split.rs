///
/// an issue raised for this repo: https://github.com/ideawu/FUCK_TCP
///
use std::io::prelude::*;
use std::net::TcpStream;
use std::string::String;
use std::collections::vec_deque::VecDeque;
use std::vec::Vec;

///
/// Can not run independently
/// need to be embedded into a console application
///
//TODO: add unit test
fn packet_split() {

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        println!("Usage: ./main ip port");
        return;
    }

    let addr = format!("{}:{}", args[1], args[2]);
    let mut stream = TcpStream::connect(&addr).unwrap();
    println!("connected to {}", &addr);

    let mut buf = [0;128];
    let mut packet_length = 0usize;
    let mut deque: VecDeque<u8> = VecDeque::new();

    loop {
        match stream.read(&mut buf) {
            Ok(n) if n == 0 => {
                println!("receive 0, exit.");
                break;
            },
            Ok(n) => {
                deque.extend(buf[0..n].iter());
                if packet_length == 0 {
                    // consume from deque to calculate packet length
                    while let Some(b) = deque.pop_front() {
                        if b == '|' as u8 {
                            break;
                        }
                        packet_length = packet_length * 10 + (b - 0x30) as usize;
                    }
                }
                if deque.len() < packet_length {
                    // incomplete packet, continue to read
                    continue;
                } else {
                    // there's at least 1 complete packet in queue, read it...
                    let bytes = deque.drain(..packet_length).collect::<Vec<u8>>();
                    let packet = String::from_utf8(bytes).unwrap();
                    println!("< {}", packet);
                    // reset packet length for next one
                    packet_length = 0;
                }
            }
            Err(e) => panic!("{:?}", e)
        }
    }
}
