use clap::{App, AppSettings, Arg, SubCommand};

pub fn get_matches() -> clap::ArgMatches<'static> {
    let mut app = App::new("Filecoin Base")
        .version("1.0")
        .about("Manage all your sectors and proofs")
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(
            Arg::with_name("config")
                .long("config")
                .short("c")
                .takes_value(true),
        )
        .subcommand(
            SubCommand::with_name("daemon").about("Starts the daemon")
                .arg(
                    Arg::with_name("prover-id")
                        .long("prover-id")
                        .help("The id of the prover, encoded as hex (31 bytes)")
                        .takes_value(true)
                )
                .arg(
                    Arg::with_name("sector-size")
                        .long("sector-size")
                        .help("The sector size to use, in bytes.")
                        .takes_value(true)
                        .default_value("1024")
                )
                .arg(
                    Arg::with_name("last-used-id")
                        .long("last-used-id")
                        .help("The last used sector id")
                        .takes_value(true)
                        .default_value("0")
                )
        )
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
        );

    #[cfg(feature = "benchy")]
    {
        app = app.subcommand(benchy_cmd());
    }

    app.get_matches()
}

#[cfg(feature = "benchy")]
fn benchy_cmd() -> App<'static, 'static> {
    SubCommand::with_name("benchy")
        .about("Benchmark specific commands")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("zigzag")
                .about("Run zigzag sealing")
                .arg(
                    Arg::with_name("size")
                        .required(true)
                        .long("size")
                        .help("The data size in KB")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("m")
                        .help("The size of m")
                        .long("m")
                        .default_value("5")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("exp")
                        .help("Expansion degree")
                        .long("expansion")
                        .default_value("8")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("sloth")
                        .help("The number of sloth iterations")
                        .long("sloth")
                        .default_value("0")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("challenges")
                        .long("challenges")
                        .help("How many challenges to execute")
                        .default_value("1")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("hasher")
                        .long("hasher")
                        .help("Which hasher should be used. Available: \"pedersen\", \"sha256\", \"blake2s\" (default \"pedersen\")")
                        .default_value("pedersen")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("layers")
                        .long("layers")
                        .help("How many layers to use")
                        .default_value("10")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("no-tmp")
                        .long("no-tmp")
                        .help("Don't use a temp file for random data (write to current directory instead).")
                )
                .arg(
                    Arg::with_name("dump")
                        .long("dump")
                        .help("Dump vanilla proofs to current directory.")
                )
                .arg(
                    Arg::with_name("partitions")
                        .long("partitions")
                        .help("How many circuit partitions to use")
                        .default_value("1")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("groth")
                        .long("groth")
                        .help("Generate and verify a groth circuit proof.")
                )
                .arg(
                    Arg::with_name("no-bench")
                        .long("no-bench")
                        .help("Don't synthesize and report inputs/constraints for a circuit.")
                )
                .arg(
                    Arg::with_name("bench-only")
                        .long("bench-only")
                        .help("Don't replicate or perform Groth proving.")
                        .conflicts_with_all(&["no-bench", "groth", "extract"])
                )

                .arg(
                    Arg::with_name("circuit")
                        .long("circuit")
                        .help("Print the constraint system.")
                )
                .arg(
                    Arg::with_name("extract")
                        .long("extract")
                        .help("Extract data after proving and verifying.")
                )
                .arg(
                    Arg::with_name("taper")
                        .long("taper")
                        .help("fraction of challenges by which to taper at each layer")
                        .default_value("0.0")
                )
                .arg(
                    Arg::with_name("taper-layers")
                        .long("taper-layers")
                        .help("number of layers to taper")
                        .takes_value(true)
                )
        )
}
