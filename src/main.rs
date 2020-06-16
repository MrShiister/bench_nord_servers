use std::any::Any;

use std::net::{IpAddr, Ipv4Addr};
use async_std::task;
use public_ip::{dns, ToResolver, Resolution};
use dns_lookup::lookup_host;
// use std::io::{self, Write};
use std::process::{Command, Stdio};
  
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
                printable: format!("{}.{}.{}.{}", ipv4.octets()[0], ipv4.octets()[1], ipv4.octets()[2], ipv4.octets()[3]) 
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
    printable: String,
}

struct Stats {
    nord_server: String,
    server_ip: String,
    latency: f32,
    jitter: f32,
    packet_loss: f32,
    download: u32,
    upload: u32,
}


fn main() {

    let myipname = String::from("");
    let servername = String::from("sg467.nordvpn.com");


    // to replace with server list
    for _i in 1..4 {

        // Change to server
        // Command::new(r#"C:\Program Files (x86)\NordVPN\NordVPN.exe"#)
        //         .arg("-c")
        //         .args(&["-n", "Singapore #467"])
        //         .output()
        //         .expect("Failed to execute NordVPN.exe");

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
        let output = Command::new(r#"E:\Downloads\Programs\Windows\speedtest\speedtest.exe"#)
                            .args(&["-f", "tsv"])
                            .stderr(Stdio::null())
                            .output()
                            .expect("Failed to execute speedtest.exe");

        if output.status.success() {
            if let Ok(s) = String::from_utf8(output.stdout) {
                let stats_vec: Vec<&str> = s.split_terminator('\t').collect();

                let nord_server = String::from("sg467.nordvpn.com");
                let latency = match stats_vec.get(2) {
                    Some(value) => value,
                    None => {
                        eprintln!("Unable to retrieve 3rd element from output.");
                        continue
                    },
                };
                let jitter = match stats_vec.get(3) {
                    Some(value) => value,
                    None => {
                        eprintln!("Unable to retrieve 4th element from output.");
                        continue
                    },
                };
                let packet_loss = match stats_vec.get(4) {
                    Some(value) => value,
                    None => {
                        eprintln!("Unable to retrieve 5th element from output.");
                        continue
                    },
                };
                let download = match stats_vec.get(5) {
                    Some(value) => value,
                    None => {
                        eprintln!("Unable to retrieve 6th element from output.");
                        continue
                    },
                };
                let upload = match stats_vec.get(6) {
                    Some(value) => value,
                    None => {
                        eprintln!("Unable to retrieve 7th element from output.");
                        continue
                    },
                };

                let latency: f32 = match latency.parse() {
                    Ok(num) => num,
                    Err(_) => {
                        eprintln!("Expected float for latency!");
                        continue
                    }
                };
                let jitter: f32 = match jitter.parse() {
                    Ok(num) => num,
                    Err(_) => 0.1,
                };

                let packet_loss: f32 = match packet_loss.parse() {
                    Ok(num) => num,
                    Err(_) => {
                        eprintln!("Expected a float for latency!");
                        continue
                    }
                };

                let download: u32 = match download.parse() {
                    Ok(num) => num,
                    Err(_) => {
                        eprintln!("Expected int for latency!");
                        continue
                    }
                };

                let upload: u32 = match upload.parse() {
                    Ok(num) => num,
                    Err(_) => {
                        eprintln!("Expected int for latency!");
                        continue
                    }
                };

                let nord467 = Stats {
                    nord_server: String::from("sg467.nordvpn.com"),
                    server_ip: server.printable,
                    latency,
                    jitter,
                    packet_loss,
                    download,
                    upload,
                };
            } else {
                eprintln!("Failed to get string from stdout!");
                continue
            }
        } else {
            if let Some(code) = output.status.code() {
                eprintln!("speedtest.exe failed with code {}", code);
                continue
            } else {
                eprintln!("speedtest.exe terminated by signal.");
                break
            }
        }
        

        break
        
        // Save score
    }

}
