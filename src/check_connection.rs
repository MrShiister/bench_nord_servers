use std::any::Any;
use std::process::{Command, Stdio};
use std::net::{IpAddr, Ipv4Addr};
use std::fmt;
use async_std::task;
use public_ip::{dns, ToResolver, Resolution};
use dns_lookup::lookup_host;

pub struct IP {
    pub ip: Ipv4Addr,
    pub octets: [u8; 4],
}

impl fmt::Display for IP {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}.{}", self.octets[0], self.octets[1], self.octets[2], self.octets[3]) 
    }
}

pub struct Stats<'a> {
    pub nord_server: &'a str,
    pub server_ip: Ipv4Addr,
    pub internet_ip: Ipv4Addr,
    pub latency: f32,
    pub jitter: f32,
    pub packet_loss: f32,
    pub no_pl_data: bool,
    pub download: f32,
    pub upload: f32,
    pub game_score: f32,
    pub usage_score: f32,
}

// Returns the IP address of the argument (a host name);
// Returns the internet IP address if argument is empty.
pub fn get_ip(servername: &str) -> Option<IP> {
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

pub fn speedtest(servername: &str, server_ip: Ipv4Addr, internet_ip: Ipv4Addr) -> Option<Stats> {
    let output = Command::new(r#"E:\Downloads\Programs\Windows\speedtest\speedtest.exe"#)
                        .args(&["-f", "tsv"])
                        .stderr(Stdio::null())
                        .output()
                        .expect("Failed to execute speedtest.exe");

    if output.status.success() {
        if let Ok(s) = String::from_utf8(output.stdout) {
            let stats_vec: Vec<&str> = s.split_terminator('\t').collect();

            // let nord_server = &servername.to_string();

            let latency = match stats_vec.get(2) {
                Some(value) => value,
                None => {
                    eprintln!("Unable to retrieve 3rd element from output.");
                    return None
                },
            };
            let jitter = match stats_vec.get(3) {
                Some(value) => value,
                None => {
                    eprintln!("Unable to retrieve 4th element from output.");
                    return None
                },
            };
            let packet_loss = match stats_vec.get(4) {
                Some(value) => value,
                None => {
                    eprintln!("Unable to retrieve 5th element from output.");
                    return None
                },
            };
            let download = match stats_vec.get(5) {
                Some(value) => value,
                None => {
                    eprintln!("Unable to retrieve 6th element from output.");
                    return None
                },
            };
            let upload = match stats_vec.get(6) {
                Some(value) => value,
                None => {
                    eprintln!("Unable to retrieve 7th element from output.");
                    return None
                },
            };

            let latency: f32 = match latency.parse() {
                Ok(num) => num,
                Err(e) => {
                    eprintln!("Expected a float for latency!");
                    eprintln!("{}", e);
                    return None
                }
            };
            let jitter: f32 = match jitter.parse() {
                Ok(num) => num,
                Err(e) => {
                    eprintln!("Expected a float for jitter!");
                    eprintln!("{}", e);
                    return None
                }
            };

            let no_pl_data;
            let packet_loss: f32 = match packet_loss.parse() {
                Ok(num) => {
                    no_pl_data = false;
                    num
                },
                Err(_) => {
                    no_pl_data = true;
                    0.3
                },
            };

            let download: f32 = match download.parse() {
                Ok(num) => num,
                Err(e) => {
                    eprintln!("Expected a float for download!");
                    eprintln!("{}", e);
                    return None
                }
            };

            let upload: f32 = match upload.parse() {
                Ok(num) => num,
                Err(e) => {
                    eprintln!("Expected a float for upload!");
                    eprintln!("{}", e);
                    return None
                }
            };

            Some(Stats {
                nord_server: servername,
                server_ip,
                internet_ip,
                latency,
                jitter,
                packet_loss,
                no_pl_data,
                download,
                upload,
                usage_score: 0.0,
                game_score: 0.0,
            })
        } else {
            eprintln!("Failed to get string from stdout!");
            None
        }
    } else {
        if let Some(code) = output.status.code() {
            eprintln!("speedtest.exe failed with code {}", code);
            None
        } else {
            eprintln!("speedtest.exe terminated by signal.");
            None
        }
    }
}

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

fn resolve_server_ip(servername: &str) -> Option<IpAddr> {
    // let ip: Result<Vec<IpAddr>, std::io::Error> = lookup_host(&servername);
    if let Ok(ip) = lookup_host(&servername) {
        Some(ip[0])
    } else {
        eprintln!("Couldn't resolve server IP!");
        None
    }
}

