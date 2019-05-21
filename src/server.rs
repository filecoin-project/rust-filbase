use std::sync::{Arc, Mutex};

use filecoin_proofs::api::safe as fil_api;
use filecoin_proofs::api::sector_builder::SectorBuilder;
use futures::prelude::*;
use futures_codec::Framed;
use runtime::net::{TcpListener, TcpStream};
use sector_base::api::porep_proof_partitions::PoRepProofPartitions;
use sector_base::api::post_proof_partitions::PoStProofPartitions;
use sector_base::api::sector_class::SectorClass;
use sector_base::api::sector_size::SectorSize;

use crate::api::*;
use crate::cbor_codec::Codec;

pub const DEFAULT_PORT: usize = 9988;

pub async fn run() -> Result<(), failure::Error> {
    let mut listener = TcpListener::bind(format!("127.0.0.1:{}", DEFAULT_PORT))?;
    println!("API listening on {}", listener.local_addr()?);

    // TODO: pull from cmd
    let last_used_id = 0;
    let prover_id = [0u8; 31];
    let sector_size = 1024;

    // TODO: pull values from the config
    let porep_partitions = 1;
    let post_partitions = 2;
    let metadata_dir = "meta";
    let sealed_sector_dir = "sealed";
    let staged_sector_dir = "staged";
    let max_num_staged_sectors = 10;

    let sb = fil_api::init_sector_builder(
        SectorClass(
            SectorSize(sector_size),
            PoRepProofPartitions(porep_partitions),
            PoStProofPartitions(post_partitions),
        ),
        last_used_id,
        metadata_dir,
        prover_id,
        sealed_sector_dir,
        staged_sector_dir,
        max_num_staged_sectors,
    )?;
    let sb = Arc::new(Mutex::new(sb));

    println!("sector builder started");

    let mut incoming = listener.incoming();

    while let Some(stream) = incoming.next().await {
        runtime::spawn(handle(stream?, sb.clone())).await?;
    }

    Ok(())
}

async fn handle(stream: TcpStream, sb: Arc<Mutex<SectorBuilder>>) -> Result<(), failure::Error> {
    println!("connected");
    let mut framed = Framed::new(stream, Codec::new());

    while let Some(res) = framed.next().await {
        let res = res?;
        println!("Got: {:?}", res);
        let response = match respond(res, sb.clone()) {
            Ok(response) => response,
            Err(err) => Response::Err(format!("{:?}", err)),
        };

        framed.send(response).await?;
    }

    Ok(())
}

fn respond(res: Request, sb: Arc<Mutex<SectorBuilder>>) -> Result<Response, failure::Error> {
    let response = match res {
        // -- Post
        Request::PostGenerate {
            comm_rs,
            challenge_seed,
        } => {
            let out = fil_api::generate_post(&sb.lock().unwrap(), comm_rs, &challenge_seed)?;
            Response::PostGenerate {
                proofs: out.proofs,
                faults: out.faults,
            }
        }
        Request::PostVerify {
            sector_size,
            proof_partitions,
            comm_rs,
            challenge_seed,
            proofs,
            faults,
        } => {
            let valid = fil_api::verify_post(
                sector_size,
                proof_partitions,
                comm_rs,
                &challenge_seed,
                proofs,
                faults,
            )?;

            Response::PostVerify(valid)
        }

        // -- Seal
        Request::SealVerify {
            sector_size,
            comm_r,
            comm_d,
            comm_r_star,
            prover_id,
            sector_id,
            proof,
        } => {
            let valid = fil_api::verify_seal(
                sector_size,
                comm_r,
                comm_d,
                comm_r_star,
                &prover_id,
                &sector_id,
                proof,
            )?;

            Response::SealVerify(valid)
        }
        Request::SealAllStaged => {
            fil_api::seal_all_staged_sectors(&sb.lock().unwrap())?;
            Response::SealAllStaged
        }
        Request::SealStatus(id) => {
            let status = fil_api::get_seal_status(&sb.lock().unwrap(), id)?;
            Response::SealStatus(status)
        }

        // -- Sector
        Request::SectorSize(sector_size) => {
            let size = fil_api::get_max_user_bytes_per_staged_sector(sector_size);
            Response::SectorSize(size)
        }
        Request::SectorListSealed => {
            let list = fil_api::get_sealed_sectors(&sb.lock().unwrap())?;
            Response::SectorListSealed(list)
        }
        Request::SectorListStaged => {
            let list = fil_api::get_staged_sectors(&sb.lock().unwrap())?;
            Response::SectorListStaged(list)
        }

        // -- Piece
        Request::PieceAdd { key, amount, path } => {
            let id = fil_api::add_piece(&sb.lock().unwrap(), &key, amount, &path)?;
            Response::PieceAdd(id)
        }
        Request::PieceRead(key) => {
            let bytes = fil_api::read_piece_from_sealed_sector(&sb.lock().unwrap(), &key)?;
            Response::PieceRead(bytes)
        }
    };

    Ok(response)
}
