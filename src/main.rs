// use tracking_allocator::AllocationRegistry;
// use tracking_allocator::Allocator;

// #[global_allocator]
// static GLOBAL: Allocator<System> = Allocator::system();

// use std::env;
use std::process::ExitCode;

use clap::Parser;
use std::net::{IpAddr, SocketAddr};

use rust_device_detector::device_detector::DeviceDetector;
use rust_device_detector::http::server;

#[derive(Parser, Debug)]
/// A commandline user agent detection tool
///
/// This is a long explanation
#[command(version)]
struct Args {
    /// Run in interactive mode.
    ///
    /// In interactive mode, each stdin line will be parsed
    /// as a user agent, and we will return on stout, one single
    /// line of json as a result.
    #[arg(short = 'i', long = "interactive")]
    interactive: bool,

    /// Run as an http server.
    ///
    /// This will run the command as an http server, listening on the
    /// specified port or 8080 by default. You submit one line of user
    /// agent and you will get back a response in json.
    #[arg(short = 's', long = "server")]
    server: bool,

    /// Address to listen on, when in http server mode.
    #[arg(
        short = 'l',
        long = "listen",
        value_name = "ADDRESS",
        default_value = "127.0.0.1"
    )]
    ip: String,

    /// Port to run on, when in http server mode.
    #[arg(short = 'p', long = "port", default_value = "8080")]
    port: u16,

    #[cfg(feature = "cache")]
    /// If set, how many entries to cache in an lru cache.
    ///
    /// Each cache entry requires about 100 bytes for the value and as many bytes as
    /// the user agent and headers (if supplied) take up for the key. That might be
    /// about 200 bytes per entry.
    #[arg(short = 'c', long = "cache", default_value = None, value_name = "ENTRIES")]
    cache: Option<u64>,

    /// When in cli mode (the default) this is the user agent to parse.
    ///
    /// Always remember escape shell arguments!
    #[arg(required_unless_present_any(["interactive", "server"]))]
    useragent: Option<String>,

    /// Generate a basic test cases instead of the normal output.
    ///
    /// This is purely to make adding new test cases easier, and the output
    /// should not be relied upon for anything else. Manual tweaking may be
    /// required, and you should ensure that all tests should work with the
    /// php version of the detector.
    #[arg(long = "gen-test-case", default_value = "false")]
    gen_test_case: bool,
}

// use stats_alloc::{Region, StatsAlloc, INSTRUMENTED_SYSTEM};

// #[global_allocator]
// static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

#[tokio::main]
async fn main() -> Result<(), ExitCode> {
    //    sc_core::setup::binary_setup();

    // let reg = stats_alloc::Region::new(&INSTRUMENTED_SYSTEM);

    let args = Args::parse();
    #[cfg(not(feature = "cache"))]
    let detector = DeviceDetector::new();

    #[cfg(feature = "cache")]
    let detector = if let Some(entries) = args.cache {
        eprintln!("Cache enabled ({} entries)", entries);
        DeviceDetector::new_with_cache(entries)
    } else {
        DeviceDetector::new()
    };

    if args.interactive {
        eprintln!("Starting interactive mode");
        let mut ua = String::with_capacity(50); // may also use with_capacity if you can guess
        while std::io::stdin().read_line(&mut ua).unwrap() > 0 {
            let headers = None;

            #[cfg(feature = "cache")]
            let detection = detector
                .parse_cached(&ua.trim_end(), headers)
                .await
                .unwrap_or_else(|_| panic!("parse failed for {}", &ua));
            #[cfg(not(feature = "cache"))]
            let detection = detector
                .parse(&ua.trim_end(), headers)
                .unwrap_or_else(|_| panic!("parse failed for {}", &ua));

            if args.gen_test_case {
                println!("{}", detection.to_test_case(&ua));
            } else {
                println!("{}", detection.to_value());
            }

            ua.clear(); // clear to reuse the buffer
        }
    } else if args.server {
        eprintln!("Starting server mode");
        let ip: IpAddr = args.ip.parse().expect("valid ip address (ipv4 or ipv6)");
        let sock = SocketAddr::new(ip, args.port);

        // tokio::spawn(async move {
        // let reg: Region<'static, System> = Region::new(&GLOBAL);
        server(sock, detector).await;
        // }).await;
    } else {
        match args.useragent {
            None => {
                eprintln!("No user agent specified");
                return Ok(());
            }

            Some(ua) => {
                // eprintln!("ua: {}", ua);
                let headers = None;

                #[cfg(feature = "cache")]
                let detection = detector
                    .parse_cached(&ua, headers)
                    .await
                    .unwrap_or_else(|_| panic!("parse failed for {}", &ua));
                #[cfg(not(feature = "cache"))]
                let detection = detector
                    .parse(&ua, headers)
                    .unwrap_or_else(|_| panic!("parse failed for {}", &ua));

                if args.gen_test_case {
                    println!("{}", detection.to_test_case(&ua));
                } else {
                    println!("{}", detection.to_value());
                }
            }
        }
    }

    // let ch = reg.change();
    // println!("allocations over entire run: {:#?} remaining {}", ch, ch.bytes_allocated - ch.bytes_deallocated);
    Ok(())
}
