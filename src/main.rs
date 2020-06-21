mod check_connection;
use crate::check_connection::{get_ip, speedtest};
use std::process::Command;
use std::io::{self, Write};
use std::io::{Error};
use std::thread::sleep;
use std::time::Duration;
use ordered_float::OrderedFloat;

fn main() -> Result<(),Error> {

    const RETRIES: u8 = 15;

    struct Weight {
        latency: f32,
        jitter: f32,
        packet_loss: f32,
        download: f32,
        upload: f32,
    }
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
    let serverlist = vec!["sg467.nordvpn.com", "sg468.nordvpn.com"];

    let mut scores = vec![];

    // to replace with server list
    for servername in &serverlist {

        print!("Connecting to {}", servername);
        io::stdout().flush().unwrap();

        let servernum: u16 = servername[2..5].parse().unwrap();

        // Change to server
        Command::new(r#"C:\Program Files (x86)\NordVPN\NordVPN.exe"#)
                .arg("-c")
                .arg("-n")
                .arg(format!("Singapore #{}", servernum))
                .output()
                .expect("Failed to execute NordVPN.exe");

        sleep(Duration::new(1, 0));

        // Get Internet IP
        let mut internet = check_connection::get_ip(&myipname);

        for _i in 1..=RETRIES {
            if let None = internet {
                sleep(Duration::new(1, 0));
                print!(".");
                io::stdout().flush().unwrap();
                internet = check_connection::get_ip(&myipname);
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

        for _i in 1..=RETRIES {
            if let None = server {
                sleep(Duration::new(1, 0));
                print!(".");
                io::stdout().flush().unwrap();
                server = check_connection::get_ip(&myipname);
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
        io::stdout().flush().unwrap();
        let stats = speedtest(&servername, server.ip, internet.ip);
        if let None = stats {
            eprintln!("Failed to do speedtest!");
            continue
        }
        let stats = stats.unwrap();
        println!("Done.");

        scores.push(stats);

        // Save score
    }

    // Tabulate and print score
    // TODO implement max function (extern crate ordered_float)
    let max_dl = scores.iter().max_by_key(|s| OrderedFloat(s.download)).unwrap().download;
    let max_ul = scores.iter().max_by_key(|s| OrderedFloat(s.upload)).unwrap().upload;

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


        // print tsv
        println!("nord_server\tserver_ip\tlatency\tjitter\tpacket_loss\tdownload\tupload\tgame_score\tusage_score");
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
