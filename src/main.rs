use std::any::Any;

use std::net::IpAddr;
use async_std::task;
use public_ip::{dns, ToResolver, Resolution};
  
fn check_connection() -> Option<IpAddr> {
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

fn main() {
    let ip = check_connection();

    match ip {
        None => {
            println!("Couldn't get IP!");
            std::process::exit(1);
        },
        Some(_) => (),
    }

    let ip = ip.unwrap();

    println!("My IP is {}", ip);

}
