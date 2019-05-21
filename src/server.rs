use futures::prelude::*;
use futures_codec::Framed;
use runtime::net::{TcpListener, TcpStream};

use crate::cbor_codec::Codec;
use crate::client::{Request, Response};

pub const DEFAULT_PORT: usize = 9988;

pub async fn run() -> Result<(), failure::Error> {
    let mut listener = TcpListener::bind(format!("127.0.0.1:{}", DEFAULT_PORT))?;

    println!("API listening on {}", listener.local_addr()?);

    let mut incoming = listener.incoming();

    while let Some(stream) = incoming.next().await {
        runtime::spawn(handle(stream?)).await?;
    }

    Ok(())
}

async fn handle(stream: TcpStream) -> Result<(), failure::Error> {
    println!("connected");
    let mut framed = Framed::new(stream, Codec::new());

    while let Some(res) = framed.next().await {
        let res = res?;
        println!("Got: {:?}", res);

        match res {
            Request::SectorSize => {
                framed.send(Response::SectorSize(1024)).await?;
            }
            Request::PieceAdd(_) => {
                framed.send(Response::Ok).await?;
            }
        }
    }

    Ok(())
}
