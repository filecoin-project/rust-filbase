#![feature(async_await)]

use clap::{value_t, values_t, App, AppSettings, Arg, SubCommand};
use failure::bail;

mod api;
mod cbor_codec;
mod client;
mod server;
mod settings;

#[macro_use]
mod macros;

#[runtime::main]
async fn main() -> Result<(), failure::Error> {
    let matches = App::new("Filecoin Base")
        .version("1.0")
        .about("Manage all your sectors and proofs")
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(
            Arg::with_name("config")
                .long("config")
                .short("c")
                .takes_value(true),
        )
        .subcommand(SubCommand::with_name("daemon").about("Starts the daemon"))
        .subcommand(
            SubCommand::with_name("post")
                .setting(AppSettings::ArgRequiredElseHelp)
                .subcommand(
                    SubCommand::with_name("generate")
                        .arg(
                            Arg::with_name("comm-rs")
                                .long("comm-rs")
                                .help("A list of hex encoded comm_rs (each 32 bytes)")
                                .use_delimiter(true)
                                .takes_value(true)
                                .required(true),
                        )
                        .arg(
                            Arg::with_name("challenge-seed")
                                .long("challenge-seed")
                                .help("Hex encoded seed (32 bytes)")
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
                                .help("A list of hex encoded comm_rs (each 32 bytes)")
                                .use_delimiter(true)
                                .takes_value(true)
                                .required(true),
                        )
                        .arg(
                            Arg::with_name("challenge-seed")
                                .long("challenge-seed")
                                .help("Hex encoded seed (32 bytes)")
                                .takes_value(true)
                                .required(true),
                        )
                        .arg(
                            Arg::with_name("proofs")
                                .long("proofs")
                                .help("A list of hex encoded proofs")
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
                                .help("Hex encoded comm_r (32 bytes)")
                                .takes_value(true)
                                .required(true),
                        )
                        .arg(
                            Arg::with_name("comm-d")
                                .long("comm-d")
                                .help("Hex encoded comm_d (32 bytes)")
                                .takes_value(true)
                                .required(true),
                        )
                        .arg(
                            Arg::with_name("comm-r-star")
                                .long("comm-r-star")
                                .help("Hex encoded comm_r_star (32 bytes)")
                                .takes_value(true)
                                .required(true),
                        )
                        .arg(
                            Arg::with_name("prover-id")
                                .long("prover-id")
                                .help("Hex encoded prover ID (31 bytes)")
                                .takes_value(true)
                                .required(true),
                        )
                        .arg(
                            Arg::with_name("sector-id")
                                .long("sector-id")
                                .help("Hex encoded sector ID (31 bytes)")
                                .takes_value(true)
                                .required(true),
                        )
                        .arg(
                            Arg::with_name("proof")
                                .long("proof")
                                .help("Hex encoded proof")
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
                        .arg(Arg::with_name("SIZE").takes_value(true).required(true)),
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
                                .takes_value(true)
                                .required(true),
                        )
                        .arg(
                            Arg::with_name("amount")
                                .help("The size of the piece in bytes, if not provided the whole file is assumed.")
                                .long("amount")
                                .takes_value(true)

                        )
                        .arg(Arg::with_name("PATH").required(true)),
                )
                .subcommand(
                    SubCommand::with_name("read").arg(Arg::with_name("KEY").required(true)),
                ),
        )
        .get_matches();

    // Load settings
    if let Some(cfg_path) = matches.value_of("config") {
        println!("loading configuration from {}", cfg_path);
        settings::Settings::load_config(cfg_path);
    }

    match matches.subcommand() {
        ("daemon", _) => server::run().await,
        ("post", Some(m)) => match m.subcommand() {
            ("generate", Some(m)) => {
                let comm_rs = hex_vec_arr!(32, m, "comm-rs")?;
                let challenge_seed = hex_arr!(32, m, "challenge-seed")?;

                client::post_generate(comm_rs, challenge_seed).await
            }
            ("verify", Some(m)) => {
                let sector_size = value_t!(m, "sector-size", u64)?;
                let proof_partitions = value_t!(m, "proof-partitions", u8)?;
                let comm_rs = hex_vec_arr!(32, m, "comm-rs")?;
                let challenge_seed = hex_arr!(32, m, "challenge-seed")?;
                let proofs = hex_vec_vec!(m, "proof")?;
                let faults = values_t!(m, "faults", u64)?;

                client::post_verify(
                    sector_size,
                    proof_partitions,
                    comm_rs,
                    challenge_seed,
                    proofs,
                    faults,
                )
                .await
            }
            _ => bail!("Unknown subcommand"),
        },
        ("seal", Some(m)) => match m.subcommand() {
            ("generate", Some(_m)) => client::seal_generate().await,
            ("verify", Some(m)) => {
                let sector_size = value_t!(m, "sector-size", u64)?;
                let comm_r = hex_arr!(32, m, "comm-r")?;
                let comm_d = hex_arr!(32, m, "comm-d")?;
                let comm_r_star = hex_arr!(32, m, "comm-r-star")?;
                let prover_id = hex_arr!(31, m, "prover-id")?;
                let sector_id = hex_arr!(31, m, "sector-id")?;
                let proof = hex_vec!(m, "proof")?;

                client::seal_verify(
                    sector_size,
                    comm_r,
                    comm_d,
                    comm_r_star,
                    prover_id,
                    sector_id,
                    proof,
                )
                .await
            }
            ("status", Some(m)) => {
                let sector_id = value_t!(m, "sector-id", u64)?;
                client::seal_status(sector_id).await
            }
            _ => bail!("Unknown subcommand"),
        },
        ("sector", Some(m)) => match m.subcommand() {
            ("size", Some(m)) => {
                let size = value_t!(m, "SIZE", u64)?;
                client::sector_size(size).await
            }
            ("list-sealed", Some(_m)) => client::sector_list_sealed().await,
            ("list-staged", Some(_m)) => client::sector_list_staged().await,
            _ => bail!("Unknown subcommand"),
        },
        ("piece", Some(m)) => match m.subcommand() {
            ("add", Some(m)) => {
                let key = m.value_of("KEY").unwrap();
                let amount = m
                    .value_of("amount")
                    .map(|s| s.parse().expect("invalid size"));
                let path = m.value_of("PATH").unwrap();

                client::piece_add(key, amount, path).await
            }
            ("read", Some(m)) => {
                let key = m.value_of("KEY").unwrap();

                client::piece_read(key).await
            }
            _ => bail!("Unknown subcommand"),
        },
        _ => bail!("Unknown subcommand"),
    }
}
