use neli::{
    consts::{rtnl::Rtm, socket::*},
    rtnl::Rtmsg,
    socket,
};
use std::io::{self, BufRead};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    // Start a thread to display status updatesbased on netlink messages
    thread::spawn(|| {
        listen_and_print_status();
    });

    // Listen for click events and open iwgtk when we get one
    loop {
        let mut last_launch = Instant::now() - Duration::from_secs(1);
        for line in io::stdin().lock().lines() {
            let _ = line.expect("Error from stdin");
            if last_launch.elapsed() >= Duration::from_secs(1) {
                last_launch = Instant::now();
                let _ = Command::new("iwgtk").spawn();
            }
        }
    }
}

fn listen_and_print_status() {
    // Connect to netlink
    let mut socket = socket::NlSocketHandle::connect(NlFamily::Route, None, &[])
        .expect("Failed to open netlink socket");

    // Make netlink socket blocking
    socket
        .block()
        .expect("Failed to make netlink socket blocking");

    // Subscribe to multicast group 7, which gives route events that indicates network changes
    socket
        .add_mcast_membership(&[7])
        .expect("Failed to subscribe to multicast group 1");

    // Act as though we last printed one second ago, so we print on our first run through
    let mut last_print = Instant::now() - Duration::from_secs(1);
    loop {
        // Print again only if at least 100ms has passed
        if last_print.elapsed() >= Duration::from_millis(100) {
            // Get the SSID uwing iwgetid
            let ssid = std::str::from_utf8(
                &Command::new("iwgetid")
                    .arg("-r")
                    .output()
                    .expect("Failed to execute command to get ssid")
                    .stdout,
            )
            .expect("Failed to read stdout from child process into string")
            .trim()
            .to_owned();

            // No SSID indicates no network connectivity
            if ssid.is_empty() {
                println!("<span foreground=\"red\">󰖪 </span>No wifi");
            } else {
                println!("<span foreground=\"#00FF00\">󰖩 </span>{ssid}");
            }
            last_print = Instant::now();
        }

        // Wait for next message
        let _ = socket
            .recv::<Rtm, Rtmsg>()
            .expect("Failed to receive message");
    }
}
