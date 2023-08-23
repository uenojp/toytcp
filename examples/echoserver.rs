use std::{env, net::Ipv4Addr, process};

use anyhow::Result;

use toytcp::tcp::TcpStream;

fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let args = env::args().collect::<Vec<_>>();

    if args.len() != 3 {
        eprintln!("Usage: {} <LOCAL IP> <LOCAL PORT>", args[0]);
        process::exit(1);
    }

    let local_address = args[1].parse::<Ipv4Addr>()?;
    let local_port = args[2].parse::<u16>()?;

    let server = TcpStream::new();
    let listning_socket = server.listen(local_address, local_port)?;

    loop {
        let _connected_socket = server.accept(listning_socket)?;
    }
}
