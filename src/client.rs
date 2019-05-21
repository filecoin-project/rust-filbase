use failure::bail;
use futures::prelude::*;
use futures_codec::Framed;
use runtime::net::TcpStream;
use serde::{Deserialize, Serialize};

use crate::cbor_codec::Codec;
use crate::server::DEFAULT_PORT;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    PieceAdd(String),
    SectorSize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    SectorSize(u64),
    Ok,
}

pub async fn piece_add(path: &str) -> Result<(), failure::Error> {
    send(Request::PieceAdd(path.into())).await?;

    Ok(())
}

pub async fn sector_size() -> Result<(), failure::Error> {
    let response = send(Request::SectorSize).await?;

    match response {
        Response::SectorSize(size) => println!("{}", size),
        _ => bail!("Invalid server response"),
    }

    Ok(())
}

async fn send<'a>(msg: Request) -> Result<Response, failure::Error> {
    let stream = TcpStream::connect(format!("127.0.0.1:{}", DEFAULT_PORT)).await?;
    let mut framed = Framed::new(stream, Codec::new());

    framed.send(msg).await?;

    let response = framed.next().await.unwrap()?;

    Ok(response)
}
