use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

// use syslog_rfc5424::SyslogMessage;
use logsnarf_rs::decoder;
use logsnarf_rs::LogData;

fn main() {
    let args: Vec<String> = env::args().collect();

    let filename: String = args[1].clone();

    let mut counter: u64 = 0;

    // File must exist in current path before this produces output
    if let Ok(lines) = read_lines(filename) {
        // Consumes the iterator, returns an (Optional) String
        for line in lines {
            if let Ok(ip) = line {
                // println!("line: {:?}", ip);
                let res = ip.parse::<LogData>();
                match res {
                    Ok(log_data) => {
                        // println!("line: {:?}", ip);
                        // println!("{:?}", log_data);
                        match decoder::decode(&log_data) {
                            Some(metric) => {
                                counter += 1;
                                // println!("{}", ip);
                                // println!("Metric: {:?}", metric);
                            }
                            None => {}
                        }
                    }
                    Err(err) => {
                        println!("Parse error! {:?}", err);
                        println!("line: {:?}", ip);
                        panic!(err);
                    }
                }
            }
        }
    }

    println!("parsed {} metrics", counter);
}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
