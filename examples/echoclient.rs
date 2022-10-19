use std::{env, net::Ipv4Addr};

use anyhow::Result;

use toytcp::Tcp;

fn main() -> Result<()> {
    let args = env::args().collect::<Vec<String>>();
    let addr = args[1].parse::<Ipv4Addr>()?;
    let port = args[2].parse::<u16>()?;
    echo_client(addr, port)?;
    Ok(())
}

fn echo_client(dst_addr: Ipv4Addr, dst_port: u16) -> Result<()> {
    let tcp = Tcp::new();
    tcp.connect(dst_addr, dst_port)?;
    Ok(())
}
