use std::env;
use std::process;
use std::time::Duration;

use rp_scale::scale::{Reading, SerialReader, SerialReaderConfig};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 6 {
        eprintln!("usage: rp-scale-read-serial <device> <baud> <unit> <max_readings> <timeout_ms>");
        process::exit(2);
    }

    let config = SerialReaderConfig::new(&args[1], parse_u32(&args[2], "baud"), &args[3]);
    let max_readings = parse_usize(&args[4], "max_readings");
    let timeout_ms = parse_u64(&args[5], "timeout_ms");
    let reader = SerialReader::new(config);

    for reading in reader.run_with_limits(max_readings, Duration::from_millis(timeout_ms)) {
        println!("{}", format_reading(&reading));
    }
}

fn format_reading(reading: &Reading) -> String {
    let weight = reading
        .weight
        .map(|weight| format!("{weight:.3}"))
        .unwrap_or_else(|| "-".to_string());
    let stable = reading
        .stable
        .map(|stable| stable.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    format!(
        "source={} port={} baud={} weight={} unit={} stable={} raw={} error={}",
        reading.source,
        reading.port,
        reading.baud,
        weight,
        reading.unit,
        stable,
        reading.raw,
        reading.error
    )
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

fn parse_usize(value: &str, name: &str) -> usize {
    value.parse::<usize>().unwrap_or_else(|_| {
        eprintln!("invalid {name}: {value}");
        process::exit(2);
    })
}
