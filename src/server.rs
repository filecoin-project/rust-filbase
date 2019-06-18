use std::sync::{Arc, Mutex};

use futures::prelude::*;
use futures_codec::Framed;
use runtime::net::{TcpListener, TcpStream};
use sector_builder::SectorBuilder;

use crate::api::*;
use crate::cbor_codec::Codec;
use crate::settings::SETTINGS;

pub async fn run(
    last_used_id: u64,
    prover_id: [u8; 31],
    sector_size: u64,
) -> Result<(), failure::Error> {
    let cfg = SETTINGS.clone().read().unwrap().clone();

    let mut listener = TcpListener::bind(cfg.server())?;
    println!("API listening on {}", listener.local_addr()?);

    let sb = SectorBuilder::init_from_metadata(
        sector_builder::SectorClass(
            sector_builder::SectorSize(sector_size),
            sector_builder::PoRepProofPartitions(cfg.porep_partitions),
            sector_builder::PoStProofPartitions(cfg.post_partitions),
        ),
        last_used_id,
        &cfg.metadata_dir,
        prover_id,
        &cfg.sealed_sector_dir,
        &cfg.staged_sector_dir,
        cfg.max_num_staged_sectors,
    )?;

    let sb = Arc::new(Mutex::new(sb));

    println!(
        "Sector builder started, with ID: {}",
        hex::encode(&prover_id)
    );

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
            let out = sb
                .lock()
                .unwrap()
                .generate_post(&comm_rs, &challenge_seed)?;
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
            let resp = filecoin_proofs::verify_post(
                filecoin_proofs::PoStConfig(
                    filecoin_proofs::SectorSize(sector_size),
                    filecoin_proofs::PoStProofPartitions(proof_partitions),
                ),
                comm_rs,
                challenge_seed,
                proofs,
                faults,
            )?;

            Response::PostVerify(resp.is_valid)
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
            let proof_partitions = porep_proof_partitions_try_from_bytes(&proof)?;

            let is_valid = filecoin_proofs::verify_seal(
                filecoin_proofs::PoRepConfig(
                    filecoin_proofs::SectorSize(sector_size),
                    proof_partitions,
                ),
                comm_r,
                comm_d,
                comm_r_star,
                &prover_id,
                &sector_id,
                &proof,
            )?;

            Response::SealVerify(is_valid)
        }
        Request::SealAllStaged => {
            sb.lock().unwrap().seal_all_staged_sectors()?;
            Response::SealAllStaged
        }
        Request::SealStatus(id) => {
            let status = sb.lock().unwrap().get_seal_status(id)?;
            Response::SealStatus(status)
        }

        // -- Sector
        Request::SectorSize(sector_size) => {
            let size = u64::from(filecoin_proofs::UnpaddedBytesAmount::from(
                filecoin_proofs::PaddedBytesAmount(sector_size),
            ));
            Response::SectorSize(size)
        }
        Request::SectorListSealed => {
            let list = sb.lock().unwrap().get_sealed_sectors()?;
            Response::SectorListSealed(list)
        }
        Request::SectorListStaged => {
            let list = sb.lock().unwrap().get_staged_sectors()?;
            Response::SectorListStaged(list)
        }

        // -- Piece
        Request::PieceAdd { key, amount, path } => {
            let destination_sector_id = &sb.lock().unwrap().add_piece(key, amount, path)?;
            Response::PieceAdd(*destination_sector_id)
        }
        Request::PieceRead(key) => {
            let piece_bytes = sb.lock().unwrap().read_piece_from_sealed_sector(key)?;
            Response::PieceRead(piece_bytes)
        }
    };

    Ok(response)
}

fn porep_proof_partitions_try_from_bytes(
    proof: &[u8],
) -> Result<filecoin_proofs::PoRepProofPartitions, failure::Error> {
    let n = proof.len();

    ensure!(
        n % filecoin_proofs::SINGLE_PARTITION_PROOF_LEN == 0,
        "no PoRepProofPartitions mapping for {:x?}",
        proof
    );

    Ok(filecoin_proofs::PoRepProofPartitions(
        (n / filecoin_proofs::SINGLE_PARTITION_PROOF_LEN) as u8,
    ))
}
