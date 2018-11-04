///
/// inspired by this repo: https://github.com/ideawu/c1000k
///
use std::env;
use std::net::{SocketAddr, TcpListener, TcpStream, Ipv4Addr};
use std::io;
use std::str::FromStr;

///
/// Can not run independently
/// need to be embedded into a console application
///
//TODO: add unit test
fn test_max_tcp_connections() -> io::Result<()> {

    let args: Vec<String> = env::args().collect();
    match args.len() {
        3 => {
            // assume it's client
            let addr = SocketAddr::from_str(format!("{}:{}", args[1], args[2]).as_str()).unwrap();
            client(addr)
        },
        2 => {
            // assume it's server
            let port: u16 = args[1].parse().unwrap();
            let addr = SocketAddr::from(([127, 0, 0, 1], port));
            let listener = TcpListener::bind(addr)?;
            println!("Server is listening to {}", addr);
            server(listener)
        },
        _ => {
            println!("Usage: {} port | {} ip port", args[0], args[0]);
            Ok(())
        }
    }
}

fn client(server_addr: SocketAddr) -> io::Result<()> {
    let mut conns = Vec::new();
    loop {
        match TcpStream::connect(server_addr) {
            Ok(stream) => conns.push(stream),
            Err(e) => break,
        }
    }
    println!("connection created: {}", conns.len());
    Ok(())
}

fn server(listener: TcpListener) -> io::Result<()> {
    let mut conns = Vec::new();
    for res in listener.incoming() {
        match res {
            Ok(stream) => conns.push(stream),
            Err(e) => break,
        }
    }
    println!("connection accepted: {}", conns.len());
    Ok(())
}

