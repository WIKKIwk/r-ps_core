use std::env;
use std::process;
use std::time::Duration;

use rp_scale::scale::{ScaleProbe, SerialPortProbe};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 5 {
        eprintln!("usage: rp-scale-probe-serial <device> <baud> <timeout_ms> <unit>");
        process::exit(2);
    }

    let device = &args[1];
    let baud = parse_u32(&args[2], "baud");
    let timeout_ms = parse_u64(&args[3], "timeout_ms");
    let unit = &args[4];

    let mut probe = SerialPortProbe::new(Duration::from_millis(timeout_ms), unit);
    match probe.probe(device, baud) {
        Ok(outcome) => {
            println!(
                "parsed_weight={} has_data={}",
                outcome.parsed_weight, outcome.has_data
            );
        }
        Err(err) => {
            eprintln!("{err}");
            process::exit(1);
        }
    }
}

fn parse_u32(value: &str, name: &str) -> u32 {
    value.parse::<u32>().unwrap_or_else(|_| {
        eprintln!("invalid {name}: {value}");
        process::exit(2);
    })
}

fn parse_u64(value: &str, name: &str) -> u64 {
    value.parse::<u64>().unwrap_or_else(|_| {
        eprintln!("invalid {name}: {value}");
        process::exit(2);
    })
}
