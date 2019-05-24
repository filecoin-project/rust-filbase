use std::fs::{File, OpenOptions};
use std::time::{Duration, Instant};
use std::u32;

use bellperson::Circuit;
use chrono::Utc;
use clap::value_t;
use failure::bail;
use fil_sapling_crypto::jubjub::JubjubBls12;
use memmap::MmapMut;
use memmap::MmapOptions;
use paired::bls12_381::Bls12;
use rand::{Rng, SeedableRng, XorShiftRng};
use storage_proofs::circuit::metric::*;
use storage_proofs::circuit::zigzag::ZigZagCompound;
use storage_proofs::compound_proof::{self, CompoundProof};
use storage_proofs::drgporep;
use storage_proofs::drgraph::*;
use storage_proofs::hasher::{Blake2sHasher, Hasher, PedersenHasher, Sha256Hasher};
use storage_proofs::layered_drgporep::{self, ChallengeRequirements, LayerChallenges};
use storage_proofs::porep::PoRep;
use storage_proofs::proof::ProofScheme;
use storage_proofs::zigzag_drgporep::*;

fn file_backed_mmap_from_zeroes(n: usize, use_tmp: bool) -> Result<MmapMut, failure::Error> {
    let file: File = if use_tmp {
        tempfile::tempfile().unwrap()
    } else {
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(format!("./zigzag-data-{:?}", Utc::now()))
            .unwrap()
    };

    file.set_len(32 * n as u64).unwrap();

    let map = unsafe { MmapOptions::new().map_mut(&file) }?;

    Ok(map)
}

fn dump_proof_bytes<H: Hasher>(
    all_partition_proofs: &[layered_drgporep::Proof<H>],
) -> Result<(), failure::Error> {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(format!("./proofs-{:?}", Utc::now()))
        .unwrap();

    serde_json::to_writer(file, all_partition_proofs)?;

    Ok(())
}

#[derive(Debug)]
struct Params {
    samples: usize,
    data_size: usize,
    m: usize,
    expansion_degree: usize,
    sloth_iter: usize,
    layer_challenges: LayerChallenges,
    partitions: usize,
    circuit: bool,
    groth: bool,
    bench: bool,
    extract: bool,
    use_tmp: bool,
    dump_proofs: bool,
    bench_only: bool,
    hasher: String,
}

fn do_the_work<H: 'static>(params: Params) -> Result<(), failure::Error>
where
    H: Hasher,
{
    println!("zigzag: {:#?}", &params);

    let Params {
        samples,
        data_size,
        m,
        expansion_degree,
        sloth_iter,
        layer_challenges,
        partitions,
        circuit,
        groth,
        bench,
        extract,
        use_tmp,
        dump_proofs,
        bench_only,
        ..
    } = &params;
    let rng = &mut XorShiftRng::from_seed([0x3dbe6259, 0x8d313d76, 0x3237db17, 0xe5bc0654]);
    let nodes = data_size / 32;

    let replica_id: H::Domain = rng.gen();
    let sp = layered_drgporep::SetupParams {
        drg: drgporep::DrgParams {
            nodes,
            degree: *m,
            expansion_degree: *expansion_degree,
            seed: new_seed(),
        },
        sloth_iter: *sloth_iter,
        layer_challenges: layer_challenges.clone(),
    };

    let pp = ZigZagDrgPoRep::<H>::setup(&sp)?;
    let mut total_proving = Duration::new(0, 0);

    let (pub_in, priv_in, d) = if *bench_only {
        (None, None, None)
    } else {
        let mut data = file_backed_mmap_from_zeroes(nodes, *use_tmp)?;

        let start = Instant::now();
        let mut replication_duration = Duration::new(0, 0);

        let (tau, aux) = ZigZagDrgPoRep::<H>::replicate(&pp, &replica_id, &mut data, None)?;
        let pub_inputs = layered_drgporep::PublicInputs::<H::Domain> {
            replica_id,
            tau: Some(tau.simplify().into()),
            comm_r_star: tau.comm_r_star,
            k: Some(0),
        };

        let priv_inputs = layered_drgporep::PrivateInputs {
            aux,
            tau: tau.layer_taus,
        };

        replication_duration += start.elapsed();

        let time_per_byte = if *data_size > (u32::MAX as usize) {
            // Duration only supports division by u32, so if data_size (of type usize) is larger,
            // we have to jump through some hoops to get the value we want, which is duration / size.
            // Consider: x = size / max
            //           y = duration / x = duration * max / size
            //           y / max = duration * max / size * max = duration / size
            let x = *data_size as f64 / u32::MAX as f64;
            let y = replication_duration / x as u32;
            y / u32::MAX
        } else {
            replication_duration / (*data_size as u32)
        };

        println!(
            "Replication: total time: {:.04}s",
            replication_duration.as_millis() as f32 / 1000.
        );
        println!(
            "Replication: time per byte: {:.04}ms",
            time_per_byte.as_nanos() as f32 / 1000.
        );

        let start = Instant::now();
        let all_partition_proofs =
            ZigZagDrgPoRep::<H>::prove_all_partitions(&pp, &pub_inputs, &priv_inputs, *partitions)?;
        let vanilla_proving = start.elapsed();
        total_proving += vanilla_proving;

        println!(
            "Vanilla proving: {:.04}ms",
            vanilla_proving.as_nanos() as f32 / 1000.
        );

        if *dump_proofs {
            dump_proof_bytes(&all_partition_proofs)?;
        }

        let mut total_verifying = Duration::new(0, 0);
        for _ in 0..*samples {
            let start = Instant::now();
            let verified = ZigZagDrgPoRep::<H>::verify_all_partitions(
                &pp,
                &pub_inputs,
                &all_partition_proofs,
            )?;
            if !verified {
                panic!("verification failed");
            }
            total_verifying += start.elapsed();
        }

        let verifying_avg = total_verifying / *samples as u32;
        let verifying_avg = f64::from(verifying_avg.subsec_nanos()) / 1_000_000_000f64
            + (verifying_avg.as_secs() as f64);

        println!("Avg verifying: {:.04}s", verifying_avg);

        (Some(pub_inputs), Some(priv_inputs), Some(data))
    };

    if *circuit || *groth || *bench {
        total_proving += do_circuit_work(&pp, pub_in, priv_in, &params)?;
    }

    if let Some(data) = d {
        if *extract {
            let start = Instant::now();
            let decoded_data = ZigZagDrgPoRep::<H>::extract_all(&pp, &replica_id, &data)?;
            let extracting = start.elapsed();
            assert_eq!(&(*data), decoded_data.as_slice());

            println!("Extracting: {:.04}s", extracting.as_millis() as f32 / 1000.);
        }
    }

    println!(
        "Total proving: {:.04}s",
        total_proving.as_millis() as f32 / 1000.
    );

    Ok(())
}

fn do_circuit_work<H: 'static + Hasher>(
    pp: &<ZigZagDrgPoRep<H> as ProofScheme>::PublicParams,
    pub_in: Option<<ZigZagDrgPoRep<H> as ProofScheme>::PublicInputs>,
    priv_in: Option<<ZigZagDrgPoRep<H> as ProofScheme>::PrivateInputs>,
    params: &Params,
) -> Result<Duration, failure::Error> {
    let mut proving_time = Duration::new(0, 0);
    let Params {
        samples,
        partitions,
        circuit,
        groth,
        bench,
        ..
    } = params;

    let engine_params = JubjubBls12::new();
    let compound_public_params = compound_proof::PublicParams {
        vanilla_params: pp.clone(),
        engine_params: &engine_params,
        partitions: Some(*partitions),
    };

    if *bench || *circuit {
        let mut cs = MetricCS::<Bls12>::new();
        ZigZagCompound::blank_circuit(&pp, &engine_params).synthesize(&mut cs)?;

        println!("circuit_num_inputs: {}", cs.num_inputs());
        println!("circuit_num_constraints: {}", cs.num_constraints());

        if *circuit {
            println!("{}", cs.pretty_print());
        }
    }

    if *groth {
        let pub_inputs = pub_in.expect("missing public inputs");
        let priv_inputs = priv_in.expect("missing private inputs");

        // TODO: The time measured for Groth proving also includes parameter loading (which can be long)
        // and vanilla proving, which may also be.
        // For now, analysis should note and subtract out these times.
        // We should implement a method of CompoundProof, which will skip vanilla proving.
        // We should also allow the serialized vanilla proofs to be passed (as a file) to the example
        // and skip replication/vanilla-proving entirely.
        let gparams =
            ZigZagCompound::groth_params(&compound_public_params.vanilla_params, &engine_params)?;

        let multi_proof = {
            let start = Instant::now();
            let result = ZigZagCompound::prove(
                &compound_public_params,
                &pub_inputs,
                &priv_inputs,
                &gparams,
            )?;
            let groth_proving = start.elapsed();
            proving_time += groth_proving;
            result
        };

        let verified = {
            let mut total_groth_verifying = Duration::new(0, 0);
            let mut result = true;
            for _ in 0..*samples {
                let start = Instant::now();
                let cur_result = result;
                ZigZagCompound::verify(
                    &compound_public_params,
                    &pub_inputs,
                    &multi_proof,
                    &ChallengeRequirements {
                        minimum_challenges: 1,
                    },
                )?;
                // If one verification fails, result becomes permanently false.
                result = result && cur_result;
                total_groth_verifying += start.elapsed();
            }
            let avg_groth_verifying = total_groth_verifying / *samples as u32;
            println!(
                "Avg groth verifying: {:.04}s",
                avg_groth_verifying.as_millis() as f32 / 1000.
            );
            result
        };
        assert!(verified);
    }

    Ok(proving_time)
}

pub fn zigzag_cmd(matches: &clap::ArgMatches) -> Result<(), failure::Error> {
    let taper = value_t!(matches, "taper", f64)?;
    let layers = value_t!(matches, "layers", usize)?;
    let taper_layers = value_t!(matches, "taper-layers", usize).unwrap_or(layers);
    let challenge_count = value_t!(matches, "challenges", usize)?;
    let layer_challenges = if taper == 0.0 {
        LayerChallenges::new_fixed(layers, challenge_count)
    } else {
        LayerChallenges::new_tapered(layers, challenge_count, taper_layers, taper)
    };

    let params = Params {
        layer_challenges,
        data_size: value_t!(matches, "size", usize)? * 1024,
        m: value_t!(matches, "m", usize)?,
        expansion_degree: value_t!(matches, "exp", usize)?,
        sloth_iter: value_t!(matches, "sloth", usize)?,
        partitions: value_t!(matches, "partitions", usize)?,
        use_tmp: !matches.is_present("no-tmp"),
        dump_proofs: matches.is_present("dump"),
        groth: matches.is_present("groth"),
        bench: !matches.is_present("no-bench") && matches.is_present("bench"),
        bench_only: matches.is_present("bench-only"),
        circuit: matches.is_present("circuit"),
        extract: matches.is_present("extract"),
        hasher: value_t!(matches, "hasher", String)?,
        samples: 5,
    };

    match params.hasher.as_ref() {
        "pedersen" => do_the_work::<PedersenHasher>(params),
        "sha256" => do_the_work::<Sha256Hasher>(params),
        "blake2s" => do_the_work::<Blake2sHasher>(params),
        _ => bail!("invalid hasher: {}", params.hasher),
    }
}
