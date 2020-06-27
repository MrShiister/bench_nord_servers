# Bench Nord Servers

## Description

This is an executable program to find the NordVPN server that gives you the best connection. In particular, this program will do a speedtest benchmark on all NordVPN servers in `serverlist.txt` and print the best server for gaming and the best server for general usage.

A `results_<time>.tsv` file is used to record the speedtest results for all valid NordVPN servers.

## Features and Known Issues

- This project is written in Rust, just for the sake of learning a new language. The original code was written in Python 3 but it was really buggy because server testing has lots of possible points of failure.

- The program has multithreading control to stop benchmarking after this server when you hit `Ctrl-C`. However, hitting `Ctrl-C` in the middle of a speedtest will also kill the speedtest.

- There are 2 very basic Rust tests written to make sure that getting the IPs work.
- The program accepts (exactly) one argument: a text file of the list of NordVPN servers in the format `llddd.nordvpn.com`. If no argument is supplied, `serverlist.txt` in the same path as the executable is loaded.
- The Windows NordVPN CLI does not have a way to verify you are indeed connected to the server you indicate. Hence, your connectivity to the correct server is verified by comparing the IP address of your Internet IP and the server IP; the first 3 octets must match, and the last octet must differ by at most 5.
- Retrieving your Internet IP address is retried 100 times (no delay) if it failed. On the basis that there is no active Internet connection in this server, failing to retrieve your Internet IP address will also stop the attempt to get the IP address of the NordVPN server and move on to the next server.



## Calculating a best server

The overall score of a server is calculated by weightage of individual stats.

### Stat Scoring

The score of each individual stat is the same for both gaming and general usage. It is calculated as follows:

| Stat                          | Formula                        |
| ----------------------------- | ------------------------------ |
| Latency (ms)                  | 1 - ()/250                     |
| Jitter (ms)                   | 1 - ()/5                       |
| Packet Loss (%)               | 1 - ()/3                       |
| Download Speed (bytes/second) | ()/max download of all servers |
| Upload Speed (bytes/second)   | ()/max upload of all servers   |

Note that not all speedtest servers provide a packet loss stat. They will be indicated as "N/A", and their packet loss score defaults to 0.9.

### Weightage

The score for a server is calculated by weightage of each individual stats. The weightage is as follows:

| Stat           | Game | General Usage |
| -------------- | ---- | ------------- |
| Latency        | 50   | 10            |
| Jitter         | 15   | 10            |
| Packet Loss    | 25   | 5             |
| Download Speed | 5    | 50            |
| Upload Speed   | 5    | 25            |
| Total          | 100  | 100           |

## Future Work

- [ ] To not let speedtest capture `SIGINT`
- [ ] To not hardcode speedtest and nordvpn executable paths