#![feature(async_await)]

use clap::{value_t, App, AppSettings, Arg, SubCommand};
use failure::bail;

mod api;
mod cbor_codec;
mod client;
mod server;

#[runtime::main]
async fn main() -> Result<(), failure::Error> {
    let matches = App::new("Filecoin Base")
        .version("1.0")
        .about("Manage all your sectors and proofs")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(SubCommand::with_name("daemon").about("Starts the daemon"))
        .subcommand(
            SubCommand::with_name("post")
                .setting(AppSettings::ArgRequiredElseHelp)
                .subcommand(
                    SubCommand::with_name("generate")
                        .arg(
                            Arg::with_name("comm-rs")
                                .long("comm-rs")
                                .help("A list of base64 encoded comm_rs (each 32 bytes)")
                                .use_delimiter(true)
                                .takes_value(true)
                                .required(true),
                        )
                        .arg(
                            Arg::with_name("challenge-seed")
                                .long("challenge-seed")
                                .help("Base64 encoded seed (32 bytes)")
                                .takes_value(true)
                                .required(true),
                        ),
                )
                .subcommand(
                    SubCommand::with_name("verify")
                        .arg(
                            Arg::with_name("sector-size")
                                .long("sector-size")
                                .takes_value(true)
                                .required(true),
                        )
                        .arg(
                            Arg::with_name("comm-rs")
                                .long("comm-rs")
                                .help("A list of base64 encoded comm_rs (each 32 bytes)")
                                .use_delimiter(true)
                                .takes_value(true)
                                .required(true),
                        )
                        .arg(
                            Arg::with_name("challenge-seed")
                                .long("challenge-seed")
                                .help("Base64 encoded seed (32 bytes)")
                                .takes_value(true)
                                .required(true),
                        )
                        .arg(
                            Arg::with_name("proofs")
                                .long("proofs")
                                .help("A list of base64 encoded proofs")
                                .use_delimiter(true)
                                .takes_value(true)
                                .required(true),
                        )
                        .arg(
                            Arg::with_name("faults")
                                .long("faults")
                                .help("A list of sector ids who faulted")
                                .use_delimiter(true)
                                .takes_value(true)
                                .required(true),
                        ),
                ),
        )
        .subcommand(
            SubCommand::with_name("seal")
                .setting(AppSettings::ArgRequiredElseHelp)
                .subcommand(SubCommand::with_name("generate"))
                .subcommand(
                    SubCommand::with_name("verify")
                        .arg(
                            Arg::with_name("sector-size")
                                .long("sector-size")
                                .takes_value(true)
                                .required(true),
                        )
                        .arg(
                            Arg::with_name("comm-r")
                                .long("comm-r")
                                .help("Base64 encoded comm_r (32 bytes)")
                                .takes_value(true)
                                .required(true),
                        )
                        .arg(
                            Arg::with_name("comm-d")
                                .long("comm-d")
                                .help("Base64 encoded comm_d (32 bytes)")
                                .takes_value(true)
                                .required(true),
                        )
                        .arg(
                            Arg::with_name("comm-r-star")
                                .long("comm-r-star")
                                .help("Base64 encoded comm_r_star (32 bytes)")
                                .takes_value(true)
                                .required(true),
                        )
                        .arg(
                            Arg::with_name("prover-id")
                                .long("prover-id")
                                .help("Base64 encoded prover ID (31 bytes)")
                                .takes_value(true)
                                .required(true),
                        )
                        .arg(
                            Arg::with_name("sector-id")
                                .long("sector-id")
                                .help("Base64 encoded sector ID (31 bytes)")
                                .takes_value(true)
                                .required(true),
                        )
                        .arg(
                            Arg::with_name("proof")
                                .long("proof")
                                .help("Base64 encoded proof")
                                .takes_value(true)
                                .required(true),
                        ),
                )
                .subcommand(
                    SubCommand::with_name("status").arg(
                        Arg::with_name("sector-id")
                            .long("sector-id")
                            .takes_value(true)
                            .required(true),
                    ),
                ),
        )
        .subcommand(
            SubCommand::with_name("sector")
                .about("Manage sectors")
                .setting(AppSettings::ArgRequiredElseHelp)
                .subcommand(
                    SubCommand::with_name("size")
                        .about("Get the size of sector")
                        .arg(
                            Arg::with_name("sector-size")
                                .long("sector-size")
                                .takes_value(true)
                                .required(true),
                        ),
                )
                .subcommand(SubCommand::with_name("list-sealed"))
                .subcommand(SubCommand::with_name("list-staged")),
        )
        .subcommand(
            SubCommand::with_name("piece")
                .about("Manage pieces")
                .setting(AppSettings::ArgRequiredElseHelp)
                .subcommand(
                    SubCommand::with_name("add")
                        .about("Add a new piece")
                        .arg(
                            Arg::with_name("key")
                                .long("key")
                                .takes_value(true)
                                .required(true),
                        )
                        .arg(
                            Arg::with_name("amount")
                                .long("amount")
                                .takes_value(true)
                                .required(true),
                        )
                        .arg(Arg::with_name("PATH").required(true)),
                )
                .subcommand(
                    SubCommand::with_name("read").arg(Arg::with_name("PATH").required(true)),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        ("daemon", _) => server::run().await,
        ("post", Some(m)) => match m.subcommand() {
            ("generate", Some(m)) => unimplemented!(),
            ("verify", Some(m)) => unimplemented!(),
            _ => bail!("Unknown subcommand"),
        },
        ("seal", Some(m)) => match m.subcommand() {
            ("generate", Some(m)) => unimplemented!(),
            ("verify", Some(m)) => unimplemented!(),
            ("status", Some(m)) => unimplemented!(),
            _ => bail!("Unknown subcommand"),
        },
        ("sector", Some(m)) => match m.subcommand() {
            ("size", Some(m)) => {
                let size = value_t!(m, "sector-size", u64)?;
                client::sector_size(size).await
            }
            ("list-sealed", Some(m)) => unimplemented!(),
            ("list-staged", Some(m)) => unimplemented!(),
            _ => bail!("Unknown subcommand"),
        },
        ("piece", Some(m)) => match m.subcommand() {
            ("add", Some(m)) => {
                let key = m.value_of("key").unwrap();
                let amount = value_t!(m, "amount", u64)?;
                let path = m.value_of("PATH").unwrap();

                client::piece_add(key, amount, path).await
            }
            ("read", Some(m)) => unimplemented!(),
            _ => bail!("Unknown subcommand"),
        },
        _ => bail!("Unknown subcommand"),
    }
}
