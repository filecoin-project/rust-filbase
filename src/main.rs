#![feature(async_await)]

use clap::{App, AppSettings, Arg, SubCommand};
use failure::bail;

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
            SubCommand::with_name("sector")
                .about("Manage sectors")
                .setting(AppSettings::ArgRequiredElseHelp)
                .subcommand(SubCommand::with_name("size").about("Get the size of sector")),
        )
        .subcommand(
            SubCommand::with_name("piece")
                .about("Manage pieces")
                .setting(AppSettings::ArgRequiredElseHelp)
                .subcommand(
                    SubCommand::with_name("add")
                        .about("Add a new piece")
                        .arg(Arg::with_name("PATH").required(true).index(1)),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        ("daemon", _) => server::run().await,
        ("sector", Some(m)) => match m.subcommand() {
            ("size", _) => client::sector_size().await,
            _ => bail!("Unknown subcommand"),
        },
        ("piece", Some(m)) => match m.subcommand() {
            ("add", Some(m)) => {
                let path = m.value_of("PATH").unwrap();
                client::piece_add(path).await
            }
            _ => bail!("Unknown subcommand"),
        },
        _ => bail!("Unknown subcommand"),
    }
}
