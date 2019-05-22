use failure::{bail, format_err, Error};
use futures::prelude::*;
use futures_codec::Framed;
use runtime::net::TcpStream;

use crate::api::*;
use crate::cbor_codec::Codec;
use crate::server::DEFAULT_PORT;

pub async fn post_generate(comm_rs: Vec<[u8; 32]>, challenge_seed: [u8; 32]) -> Result<(), Error> {
    let res = send(Request::PostGenerate {
        comm_rs,
        challenge_seed,
    })
    .await?;

    match res {
        Response::PostGenerate { proofs, faults } => {
            println!("Proofs");
            for proof in &proofs {
                println!("{}", hex::encode(proof));
            }

            println!("Faults");
            println!("{:?}", faults);
        }
        _ => bail!("Invalid server response"),
    }

    Ok(())
}

pub async fn post_verify(
    sector_size: u64,
    proof_partitions: u8,
    comm_rs: Vec<[u8; 32]>,
    challenge_seed: [u8; 32],
    proofs: Vec<Vec<u8>>,
    faults: Vec<u64>,
) -> Result<(), Error> {
    let res = send(Request::PostVerify {
        sector_size,
        proof_partitions,
        comm_rs,
        challenge_seed,
        proofs,
        faults,
    })
    .await?;

    match res {
        Response::PostVerify(valid) => {
            println!("{}", valid);
        }
        _ => bail!("Invalid server response"),
    }

    Ok(())
}

pub async fn seal_generate() -> Result<(), Error> {
    let res = send(Request::SealAllStaged).await?;
    match res {
        Response::SealAllStaged => {}
        _ => bail!("Invalid server response"),
    }

    Ok(())
}

pub async fn seal_verify(
    sector_size: u64,
    comm_r: [u8; 32],
    comm_d: [u8; 32],
    comm_r_star: [u8; 32],
    prover_id: [u8; 31],
    sector_id: [u8; 31],
    proof: Vec<u8>,
) -> Result<(), Error> {
    let res = send(Request::SealVerify {
        sector_size,
        comm_r,
        comm_d,
        comm_r_star,
        prover_id,
        sector_id,
        proof,
    })
    .await?;

    match res {
        Response::SealVerify(valid) => {
            println!("{}", valid);
        }
        _ => bail!("Invalid server response"),
    }

    Ok(())
}

pub async fn seal_status(sector_id: u64) -> Result<(), Error> {
    let res = send(Request::SealStatus(sector_id)).await?;
    match res {
        Response::SealStatus(status) => {
            println!("{:?}", status);
        }
        _ => bail!("Invalid server response"),
    }

    Ok(())
}

pub async fn sector_size(size: u64) -> Result<(), Error> {
    let response = send(Request::SectorSize(size)).await?;

    match response {
        Response::SectorSize(size) => println!("{}", size),
        _ => bail!("Invalid server response"),
    }

    Ok(())
}

pub async fn sector_list_sealed() -> Result<(), Error> {
    let response = send(Request::SectorListSealed).await?;
    match response {
        Response::SectorListSealed(list) => {
            for el in &list {
                println!("{:?}", el);
            }
        }
        _ => bail!("Invalid server response"),
    }

    Ok(())
}
pub async fn sector_list_staged() -> Result<(), Error> {
    let response = send(Request::SectorListStaged).await?;
    match response {
        Response::SectorListStaged(list) => {
            for el in &list {
                println!("{:?}", el);
            }
        }
        _ => bail!("Invalid server response"),
    }

    Ok(())
}

pub async fn piece_add<S1: AsRef<str>, S2: AsRef<str>>(
    key: S1,
    amount: u64,
    path: S2,
) -> Result<(), Error> {
    let res = send(Request::PieceAdd {
        key: key.as_ref().into(),
        amount,
        path: path.as_ref().into(),
    })
    .await?;

    match res {
        Response::PieceAdd(id) => println!("{}", id),
        _ => bail!("Invalid server response"),
    }

    Ok(())
}

pub async fn piece_read<S: AsRef<str>>(key: S) -> Result<(), Error> {
    let response = send(Request::PieceRead(key.as_ref().into())).await?;

    match response {
        Response::PieceRead(bytes) => println!("{}", hex::encode(bytes)),
        _ => bail!("Invalid server response"),
    }

    Ok(())
}

async fn send<'a>(msg: Request) -> Result<Response, Error> {
    let stream = TcpStream::connect(format!("127.0.0.1:{}", DEFAULT_PORT)).await?;
    let mut framed = Framed::new(stream, Codec::new());

    framed.send(msg).await?;

    let response = framed.next().await.unwrap()?;
    match response {
        _ => Ok(response),
        Response::Err(err) => Err(format_err!("Server error: {}", err)),
    }
}
