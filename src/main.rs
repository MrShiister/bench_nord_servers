use std::any::Any;

use std::net::{IpAddr, Ipv4Addr};
use async_std::task;
use public_ip::{dns, ToResolver, Resolution};
use dns_lookup::lookup_host;
use std::process::Command;
  
fn resolve_internet_ip() -> Option<IpAddr> {
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

fn resolve_server_ip(servername: &String) -> Option<IpAddr> {
    // let ip: Result<Vec<IpAddr>, std::io::Error> = lookup_host(&servername);
    if let Ok(ip) = lookup_host(&servername) {
        Some(ip[0])
    } else {
        eprintln!("Couldn't resolve server IP!");
        None
    }
}

fn build_ip_struct(servername: &String) -> Option<IP> {
    let wrapped;

    if servername.is_empty() {
        wrapped = resolve_internet_ip();
    } else {
        wrapped = resolve_server_ip(&servername);
    }
    
    if let Some(ip) = wrapped {
        if let IpAddr::V4(ipv4) = ip {
            Some(IP {
                ip: ipv4,
                octets: ipv4.octets(),
            })
        } else {
            println!("Not handling IPv6!");
            None
        }
    } else {
        println!("Couldn't resolve Internet IP!");
        None
    }
}

struct IP {
    ip: Ipv4Addr,
    octets: [u8; 4],
}


fn main() {

    let myipname = String::from("");
    let servername = String::from("sg467.nordvpn.com");


    // to replace with server list
    for _i in 1..4 {

        // Change to server
        Command::new(r#"C:\Program Files (x86)\NordVPN\NordVPN.exe"#)
                .arg("-c")
                .args(&["-n", "Singapore #467"])
                .output()
                .expect("Failed to execute NordVPN.exe");

        // Get Internet IP
        let internet = build_ip_struct(&myipname);

        if let None = internet {
            eprintln!("Failed to get internet IP!");
            continue
        }

        let internet = internet.unwrap();
        println!("    My IP is {}", internet.ip);

        // Get Server IP
        let server = build_ip_struct(&servername);
        if let None = server {
            eprintln!("Failed to get server IP!");
            continue
        }
        let server = server.unwrap();
        println!("Server IP is {}", server.ip);

        if internet.octets[0] != server.octets[0] ||
            internet.octets[1] != server.octets[1] ||
            internet.octets[2] != server.octets[2] ||
            internet.octets[3] - server.octets[3] > 5 {
            println!("IP Mismatch.");
            continue
        }

        // Speedtest
        
        // Save score
    }

}
