use std::{env, net::Ipv4Addr, process};

use anyhow::Result;

use toytcp::tcp::TcpStream;

fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let args = env::args().collect::<Vec<_>>();

    if args.len() != 3 {
        eprintln!("Usage: {} <REMOTE IP> <REMOTE PORT>", args[0]);
        process::exit(1);
    }

    let remote_address = args[1].parse::<Ipv4Addr>()?;
    let remote_port = args[2].parse::<u16>()?;

    let client = TcpStream::new();
    client.connect(remote_address, remote_port)?;

    Ok(())
}
