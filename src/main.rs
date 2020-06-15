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
        //println!("Couldn't get an IP address.");
        None
    }

}

fn get_server_ip(hostname: &String) -> Option<IpAddr> {
    let ip: Result<Vec<IpAddr>, std::io::Error> = lookup_host(&hostname);

    match ip {
        Ok(ip) => Some(ip[0]),
        Err(error) => {
            println!("{}", error);
            None
        },
    }
}

fn main() {

    struct IP {
        ip: IpAddr,
        octets: [u8; 4],
    }

    let mut wrapped;
    let mut ip;
    let mut octets;

    let hostname = String::from("sg467.nordvpn.com");

    for _i in 1..4 {

        // Get Internet IP

        wrapped = get_internet_ip();

        match wrapped {
            None => {
                println!("Couldn't get Internet IP!");
                continue
                //std::process::exit(1);
            },
            Some(_) => (),
        }

        ip = wrapped.unwrap();

        octets = match ip {
            IpAddr::V4(ipv4) => ipv4.octets(),
            _ => {
                println!("Not handling IPv6!");
                continue
            },
        };

        let internet = IP {
            ip,
            octets,
        };
        println!("    My IP is {}", internet.ip);


        // Get Server IP
        wrapped = get_server_ip(&hostname);

        match wrapped {
            None => {
                println!("Couldn't get Server IP!");
                continue
            },
            Some(_) => (),
        }

        ip = wrapped.unwrap();

        octets = match ip {
            IpAddr::V4(ipv4) => ipv4.octets(),
            _ => {
                println!("Not handling IPv6!");
                continue
            },
        };
        let server = IP {
            ip,
            octets,
        };
        println!("Server IP is {}", server.ip);
    }

}
