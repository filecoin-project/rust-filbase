#![feature(async_await)]

#[macro_use]
extern crate failure;
extern crate filecoin_proofs;
#[cfg(feature = "benchy")]
#[macro_use]
extern crate prometheus;
extern crate sector_builder;

use clap::{value_t, values_t};
use failure::bail;
use rand::{thread_rng, Rng};

mod api;
mod app;
#[cfg(feature = "benchy")]
mod benchy;
mod cbor_codec;
mod client;
mod server;
mod settings;

#[macro_use]
mod macros;

#[runtime::main]
async fn main() -> Result<(), failure::Error> {
    let matches = app::get_matches();

    // Load settings
    if let Some(cfg_path) = matches.value_of("config") {
        println!("loading configuration from {}", cfg_path);
        settings::Settings::load_config(cfg_path);
    }

    match matches.subcommand() {
        ("daemon", Some(m)) => {
            let prover_id = if m.value_of("prover-id").is_some() {
                hex_arr!(31, m, "prover-id")?
            } else {
                let mut rng = thread_rng();
                rng.gen()
            };
            let last_used_id = value_t!(m, "last-used-id", u64)?;
            let sector_size = value_t!(m, "sector-size", u64)?;

            server::run(last_used_id, prover_id, sector_size).await
        }
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
                let proofs = hex_vec_vec!(m, "proofs")?;
                let faults = values_t!(m, "faults", u64).unwrap_or_else(|_| vec![]);

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
                    .map(|s| s.parse().expect("invalid size"))
                    .unwrap();
                let path = m.value_of("PATH").unwrap();

                client::piece_add(key, amount, path).await
            }
            ("read", Some(m)) => {
                let key = m.value_of("KEY").unwrap();

                client::piece_read(key).await
            }
            _ => bail!("Unknown subcommand"),
        },
        ("benchy", Some(_)) => {
            #[cfg(not(feature = "benchy"))]
            bail!("Please compile with the benchy feature flag to enable benchmarking");
            #[cfg(feature = "benchy")]
            {
                match m.subcommand() {
                    ("zigzag", Some(m)) => benchy::zigzag::zigzag_cmd(m),
                    _ => bail!("Unknown subcommand"),
                }
            }
        }
        _ => bail!("Unknown subcommand"),
    }
}
