use failure::bail;
use futures::prelude::*;
use futures_codec::Framed;
use runtime::net::TcpStream;

use crate::api::*;
use crate::cbor_codec::Codec;
use crate::server::DEFAULT_PORT;

pub async fn piece_add<S1: AsRef<str>, S2: AsRef<str>>(
    key: S1,
    amount: u64,
    path: S2,
) -> Result<(), failure::Error> {
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

pub async fn sector_size(size: u64) -> Result<(), failure::Error> {
    let response = send(Request::SectorSize(size)).await?;

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
