use log::{info, warn, LevelFilter};
use redox_log::{OutputBuilder, RedoxLogger};

use redox_scheme::{RequestKind, SchemeMut, SignalBehavior, Socket, V2};

use scheme::{BuzzScheme};

mod scheme;

enum Ty {
    Buzz,
}

fn main() {
    let _ = RedoxLogger::new()
    .with_output(
        OutputBuilder::stdout()
            .with_filter(log::LevelFilter::Debug)
            .with_ansi_escape_codes()
            .build()
    )
    .with_process_name("buzz".into())
    .enable(); 
    info!("buzz logger started");
    
    //get arg 0 (name used to start)
    let ty = match &*std::env::args().next().unwrap() {
        "buzz" => Ty::Buzz,
        _ => panic!("needs to be called as buzz"),
    };

    redox_daemon::Daemon::new(move |daemon| {
        let name = match ty {
            Ty::Buzz => "buzz",
        };
        let socket = Socket::<V2>::create(name).expect("buzz: failed to create demo scheme");
        let mut demo_scheme= BuzzScheme(ty, 1);

        libredox::call::setrens(0, 0).expect("buzz: failed to enter null namespace");

        daemon.ready().expect("buzz: failed to notify parent");
        //the length of this buffer will be read by the dd command and written to gtdemo scheme
        let buffer = &[0; 13];
        let _ = demo_scheme.write(0, buffer, 0, 0).expect("failed to write BUZZ to /scheme/buzz");

        loop {
            info!("buzz daemon loop start");
            // dd if=/scheme/buzz of=/scheme/gtdemo count=1 
            let Some(request) = socket
                .next_request(SignalBehavior::Restart)
                .expect("buzz: failed to read events from demo scheme")
            else {
                warn!("exiting buzz");
                std::process::exit(0);
            };
            info!("request: {request:?}");

            match request.kind() {
                RequestKind::Call(request) => {
                    let response = request.handle_scheme_mut(&mut demo_scheme);

                    socket
                        .write_responses(&[response], SignalBehavior::Restart)
                        .expect("buzz: failed to write responses to demo scheme");
                }
                _ => (),
            }
            info!("running buzz daemon")
        }
    })
    .expect("buzz: failed to daemonize");
}