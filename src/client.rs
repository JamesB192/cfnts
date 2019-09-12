// This file is part of cf-nts.
// Copyright (c) 2019, Cloudflare. All rights reserved.
// See LICENSE for licensing information.

//! NTS client implementation.

use slog::debug;
use std::process;

use crate::config;
use crate::ntp::client::run_nts_ntp_client;
use crate::nts_ke::client::run_nts_ke_client;

/// The entry point of `client`.
pub fn run<'a>(matches: &clap::ArgMatches<'a>) {
    // This should return the clone of `logger` in the main function.
    let logger = slog_scope::logger();

    let host = matches
        .value_of("host")
        .map(String::from)
        .unwrap();
    let port = matches.value_of("port").map(String::from);
    let cert_file = matches.value_of("cert").map(String::from);

    // By default, use_ipv4 is None (no preference for using either ipv4 or ipv6
    // so client sniffs which one to use based on support)
    // However, if a user specifies the ipv4 flag, we set use_ipv4 = Some(true)
    // If they specify ipv6 (only one can be specified as they are mutually exclusive
    // args), set use_ipv4 = Some(false)
    let ipv4 = matches.is_present("ipv4");
    let mut use_ipv4 = None;
    if ipv4 {
        use_ipv4 = Some(true);
    } else {
        // Now need to check whether ipv6 is being used, since ipv4 has not been mandated
        if matches.is_present("ipv6") {
            use_ipv4 = Some(false);
        }
    }

    let mut trusted_cert = None;
    if let Some(file) = cert_file {
        if let Ok(certs) = config::load_tls_certs(file) {
            trusted_cert = Some(certs[0].clone());
        }
    }

    let client_config = config::ConfigNTSClient {
        host,
        port,
        trusted_cert,
        use_ipv4,
    };

    let res = run_nts_ke_client(&logger, client_config);

    match res {
        Err(err) => {
            eprintln!("failure of tls stage {:?}", err);
            process::exit(125)
        }
        Ok(_) => {}
    }
    let state = res.unwrap();
    debug!(logger, "running UDP client with state {:x?}", state);
    let res = run_nts_ntp_client(&logger, state);
    match res {
        Err(err) => {
            eprintln!("Failure of client {:?}", err);
            process::exit(126)
        }
        Ok(result) => {
            println!("stratum: {:}", result.stratum);
            println!("offset: {:.6}", result.time_diff);
            process::exit(0)
        }
    }
}
