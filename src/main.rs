use std::any::Any;

use std::net::IpAddr;
use async_std::task;
use public_ip::{dns, ToResolver, Resolution};
use dns_lookup::lookup_host;
  
fn get_internet_ip() -> Option<IpAddr> {
    // List of resolvers to try and get an IP address from
    let resolver = dns::OPENDNS_RESOLVER.to_resolver();
    // Attempt to get an IP address and print it
    if let Some(resolution) = task::block_on(public_ip::resolve(resolver)) {
        if let Some(resolution) = Any::downcast_ref::<dns::DnsResolution>(&resolution) {
            // println!("public ip address {:?} resolved from {:?} ({:?})",
            //     resolution.address(),
            //     resolution.name(),
            //     resolution.server(),
            // );
            Some(resolution.address())
        } else {
            None
        }
    } else {
        println!("Couldn't get an IP address.");
        None
    }

}

fn get_server_ip(hostname: String) -> Option<Vec<IpAddr>> {
    let ip: Result<Vec<IpAddr>, std::io::Error> = lookup_host(&hostname);

    match ip {
        Ok(ip) => Some(ip),
        Err(error) => {
            println!("{}", error);
            None
        },
    }
    // let ip: Option<Vec<std::net::IpAddr>> = lookup_host(&hostname);
}

fn main() {
    let internet_ip = get_internet_ip();

    match internet_ip {
        None => {
            println!("Couldn't get Internet IP!");
            std::process::exit(1);
        },
        Some(_) => (),
    }

    let internet_ip = internet_ip.unwrap();
    println!("My IP is {}", internet_ip);

    let hostname = String::from("sg467.nordvpn.com");
    let server_ip = get_server_ip(hostname);

    match server_ip {
        None => {
            println!("Couldn't get Server IP!");
            std::process::exit(1);
        },
        Some(_) => (),
    }

    let server_ip = server_ip.unwrap();
    println!("Server IP is {}", &server_ip[0]);

}
