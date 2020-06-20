mod check_connection;
use crate::check_connection::{return_ip, speedtest};

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
        let internet = check_connection::return_ip(&myipname);

        if let None = internet {
            eprintln!("Failed to get internet IP!");
            continue
        }

        let internet = internet.unwrap();
        println!("    My IP is {}", internet.ip);

        // Get Server IP
        let server = return_ip(&servername);
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
        let stats = speedtest(&servername);

        break
        
        // Save score
    }

}
