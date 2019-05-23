use filecoin_proofs::api::sector_builder::metadata::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    // -- Post
    PostGenerate {
        comm_rs: Vec<[u8; 32]>,
        challenge_seed: [u8; 32],
    },
    PostVerify {
        sector_size: u64,
        proof_partitions: u8,
        comm_rs: Vec<[u8; 32]>,
        challenge_seed: [u8; 32],
        proofs: Vec<Vec<u8>>,
        faults: Vec<u64>,
    },

    // -- Seal
    SealVerify {
        sector_size: u64,
        comm_r: [u8; 32],
        comm_d: [u8; 32],
        comm_r_star: [u8; 32],
        prover_id: [u8; 31],
        sector_id: [u8; 31],
        proof: Vec<u8>,
    },
    SealAllStaged,
    SealStatus(u64),

    // -- Sector
    SectorSize(u64),
    SectorListSealed,
    SectorListStaged,

    // -- Piece
    PieceAdd {
        key: String,
        amount: Option<u64>,
        path: String,
    },
    PieceRead(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    // -- Post
    PostGenerate {
        proofs: Vec<Vec<u8>>,
        faults: Vec<u64>,
    },
    PostVerify(bool),

    // -- Seal
    SealVerify(bool),
    SealAllStaged,
    SealStatus(SealStatus),

    // -- Sector
    SectorSize(u64),
    SectorListSealed(Vec<SealedSectorMetadata>),
    SectorListStaged(Vec<StagedSectorMetadata>),

    // -- Piece
    PieceAdd(u64),
    PieceRead(Vec<u8>),

    /// Used for `Err(some_error)` return types.
    Err(String),
}
