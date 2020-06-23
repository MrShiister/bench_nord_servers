use async_std::task;
use dns_lookup::lookup_host;
use ordered_float::OrderedFloat;
use public_ip::{dns, ToResolver, Resolution};
use std::{
    any::Any,
    error::Error,
    fmt,
    fs,
    io::{self, Write},
    net::{IpAddr, Ipv4Addr},
    process::{self, Command, Stdio},
    thread::sleep,
    time::Duration,
};

pub struct Config {
    filename: String,
    retries: u8,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        let filename;
        if args.len() == 1 {
            filename = String::from("serverlist.txt")
        } else if args.len() == 2 {
            filename = args[1].clone();
        } else {
            return Err("Too many arguments!");
        }

        let retries = 15;

        Ok(Config { filename, retries })
    }
}


pub struct IP {
    pub ip: Ipv4Addr,
    pub octets: [u8; 4],
}

impl fmt::Display for IP {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}.{}", self.octets[0], self.octets[1], self.octets[2], self.octets[3]) 
    }
}

struct Stats<'a> {
    nord_server: &'a str,
    server_ip: Ipv4Addr,
    internet_ip: Ipv4Addr,
    latency: f32,
    jitter: f32,
    packet_loss: f32,
    no_pl_data: bool,
    download: f32,
    upload: f32,
    game_score: f32,
    usage_score: f32,
}

struct Weight {
    latency: f32,
    jitter: f32,
    packet_loss: f32,
    download: f32,
    upload: f32,
}


pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let game_wt = Weight {
        latency: 50.0,
        jitter: 15.0,
        packet_loss: 25.0,
        download: 5.0,
        upload: 5.0,
    };
    let usage_wt = Weight {
        latency: 10.0,
        jitter: 10.0,
        packet_loss: 5.0,
        download: 50.0,
        upload: 25.0,
    };

    let myipname = String::from("");

    //let serverlist = vec!["sg467.nordvpn.com", "sg468.nordvpn.com"];
    let mut serverlist: Vec<String> = Vec::new();
    let contents = fs::read_to_string(&config.filename)?;
    for line in contents.lines() {
        serverlist.push(line.to_string());
    }

    if serverlist.len() == 0 {
        eprintln!("No servers collected! {} is empty.", config.filename);
        process::exit(1);
    }

    let mut scores = vec![];

    // to replace with server list
    for servername in &serverlist {

        println!("Connecting to {}", servername);

        let servernum: u16 = servername[2..5].parse().unwrap_or_else(|err| {
            eprintln!("Failed to read server number: {}", err);
            process::exit(1);
        });

        // Change to server
        Command::new(r#"C:\Program Files (x86)\NordVPN\NordVPN.exe"#)
                .arg("-c")
                .arg("-n")
                .arg(format!("Singapore #{}", servernum))
                .output()
                .expect("Failed to execute NordVPN.exe");

        sleep(Duration::new(1, 0));

        // Get Internet IP
        let mut internet = get_ip(&myipname);

        for _i in 1..=config.retries {
            if let None = internet {
                sleep(Duration::new(1, 0));
                internet = get_ip(&myipname);
            } else {
                break
            }
        }
         if let None = internet {
             eprintln!(": Failed to get internet IP!");
             continue
         }

        let internet = internet.unwrap();
        // println!("    My IP is {}", internet.ip);

        // Get Server IP
        let mut server = get_ip(&servername);

        for _i in 1..=config.retries {
            if let None = server {
                sleep(Duration::new(1, 0));
                server = get_ip(&myipname);
            } else {
                break
            }
        }
        if let None = server {
            eprintln!(": Failed to get server IP!");
            continue
        }
        let server = server.unwrap();
        // println!("Server IP is {}", server.ip);

        if internet.octets[0] != server.octets[0] ||
            internet.octets[1] != server.octets[1] ||
            internet.octets[2] != server.octets[2] ||
            internet.octets[3] - server.octets[3] > 5 {
            println!(": IP Mismatch.");
            continue
        } else {
            println!(": Successful.");
        }

        // Speedtest
        print!("Speedtesting... ");
        io::stdout().flush().unwrap_or_else(|err| {
            eprintln!("Failed to flush stdout: {}", err);
            process::exit(1);
        });
        let stats = speedtest(&servername, server.ip, internet.ip);
        if let None = stats {
            eprintln!("Failed to do speedtest!");
            continue
        }
        let stats = stats.unwrap();
        println!("Done.");

        scores.push(stats);

        // TODO handle Interrupt

    }

    if let Err(e) = tabulate_score(scores, game_wt, usage_wt) {
        eprintln!("Failed to tabulate scores: {}", e);
        process::exit(1);
    }

    Ok(())
}

fn tabulate_score(mut scores: Vec<Stats>, game_wt: Weight, usage_wt: Weight) -> Result<(), Box<dyn Error>> {
    // Tabulate and print score
    let max_dl = scores.iter().max_by_key(|s| OrderedFloat(s.download)).unwrap_or_else(|| {
        eprintln!("Problem finding max of download speeds");
        process::exit(1);
    }).download;
    let max_ul = scores.iter().max_by_key(|s| OrderedFloat(s.upload)).unwrap_or_else(|| {
        eprintln!("Problem finding max of upload speeds");
        process::exit(1);
    }).upload;

    if max_dl == 0.0 {
        return Err("Max download is 0".into())
    }
    if max_ul == 0.0 {
        return Err("Max upload is 0".into())
    }

    for score in &mut scores {
        // Calculate game_score and usage_score
        score.game_score =  (1.0 - score.latency/250.0)   * game_wt.latency       +
                            (1.0 - score.jitter/5.0)      * game_wt.jitter        +
                            (1.0 - score.packet_loss/3.0) * game_wt.packet_loss   +
                            score.download/max_dl         * game_wt.download      +
                            score.upload/max_ul           * game_wt.upload;

        score.usage_score =  (1.0 - score.latency/250.0)   * usage_wt.latency       +
                             (1.0 - score.jitter/5.0)      * usage_wt.jitter        +
                             (1.0 - score.packet_loss/3.0) * usage_wt.packet_loss   +
                             score.download/max_dl         * usage_wt.download      +
                             score.upload/max_ul           * usage_wt.upload;


        // sort scores

        // print tsv
        // TODO write to file
        println!("nord_server\tserver_ip\tinternet_ip\tlatency\tjitter\tpacket_loss\tdownload\tupload\tgame_score\tusage_score");
        match score.no_pl_data {
            false => {
                println!("{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                    score.nord_server,
                    score.server_ip,
                    score.internet_ip,
                    score.latency,
                    score.jitter,
                    score.packet_loss,
                    score.download,
                    score.upload,
                    score.game_score,
                    score.usage_score
                );
            }
            true => {
                println!("{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                    score.nord_server,
                    score.server_ip,
                    score.latency,
                    score.jitter,
                    "N/A",
                    score.download,
                    score.upload,
                    score.game_score,
                    score.usage_score
                );
            }
        }
        // println!("Download speed in {} is {:.1}MB/s", score.nord_server, score.download/1_000_000.0);
    }


    Ok(())

}
// Returns the IP address of the argument (a hostname);
// Returns the internet IP address if string is 0-length.
fn get_ip(servername: &str) -> Option<IP> {
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
            println!(": Not handling IPv6!");
            None
        }
    } else {
        println!(": Couldn't resolve Internet IP!");
        None
    }
}

fn speedtest(servername: &str, server_ip: Ipv4Addr, internet_ip: Ipv4Addr) -> Option<Stats> {
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

