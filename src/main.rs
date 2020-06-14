#[derive(Debug)]
enum IPs {
    Server(u8, u8, u8, u8),
    Connected(u8, u8, u8, u8),
    LastConnected(u8, u8, u8, u8),
}

fn main() {
    let server_ip = Some(IPs::Server(1,2,3,4));
    let server_ip2 = None;

    let server_ip = server_ip.unwrap_or(IPs::Server(0, 0, 0, 0));
    let server_ip2 = server_ip2.unwrap_or(IPs::Server(0, 0, 0, 0));

    let mut valid = 1;
    if let IPs::Server(0, 0, 0, 0) = server_ip {
        valid = 0;
    }

    let valid2 = match server_ip2 {
        IPs::Server(_, _, _, _) => 1,
        IPs::Connected(_, _, _, _) => 2,
        IPs::LastConnected(_, _, _, _) => 3,
    };
    println!("server_ip is {:?}", valid);
    println!("server_ip2 is {:?}", valid2);
}
