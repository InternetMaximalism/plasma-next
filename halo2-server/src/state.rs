use actix::prelude::*;
use stark_verifier::bn254_poseidon::plonky2_config::Bn254PoseidonGoldilocksConfig;
use std::{
    collections::HashMap,
    sync::RwLock,
    time::{Duration, Instant},
};
use zkp::serialization::serialized_proof::SerializedProof;

#[cfg(not(feature = "debug"))]
use plonky2::plonk::{circuit_data::CircuitData, config::GenericConfig};

#[cfg(not(feature = "debug"))]
use crate::snark_processor::{generate_proof_tuple_and_data, SnarkProcessor};

#[cfg(not(feature = "debug"))]
const D: usize = 2;
#[cfg(not(feature = "debug"))]
type C = Bn254PoseidonGoldilocksConfig;
#[cfg(not(feature = "debug"))]
type F = <C as GenericConfig<D>>::F;

/// The time-to-live for a proof in seconds
pub const PROOF_TTL: u64 = 60 * 60;

pub struct SnarkState {
    #[cfg(not(feature = "debug"))]
    data: CircuitData<F, C, D>,
    #[cfg(not(feature = "debug"))]
    snark_processor: SnarkProcessor,
}

impl SnarkState {
    #[cfg(not(feature = "debug"))]
    pub fn new() -> Self {
        let (dummy_proof_tuple, data) = generate_proof_tuple_and_data();
        let snark_processor = SnarkProcessor::load(dummy_proof_tuple.clone());
        Self {
            data,
            snark_processor,
        }
    }

    #[cfg(feature = "debug")]
    pub fn new() -> Self {
        Self {}
    }

    #[cfg(not(feature = "debug"))]
    pub fn prove(&self, proof: SerializedProof) -> String {
        let proof = proof.to_proof(&self.data).unwrap();
        self.snark_processor.prove(proof).proof
    }

    #[cfg(feature = "debug")]
    pub fn prove(&self, _proof: SerializedProof) -> String {
        log::debug!("Waiting for 1 minutes...");

        let two_minutes = Duration::from_secs(60);
        std::thread::sleep(two_minutes);

        "0xaaaa".to_string()
    }
}

/// An actor that holds state
pub struct StateActor {
    states: HashMap<Vec<u8>, RwLock<(Option<String>, Instant)>>,
}

impl StateActor {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
        }
    }
}

/// Messages sent to the actor
pub struct SetProof {
    pub key: Vec<u8>,
    pub value: String,
}

impl Message for SetProof {
    type Result = ();
}

impl Handler<SetProof> for StateActor {
    type Result = ();

    fn handle(&mut self, msg: SetProof, _: &mut Context<Self>) {
        let expire_in = PROOF_TTL;
        self.states.insert(
            msg.key,
            RwLock::new((
                Some(msg.value),
                Instant::now() + Duration::from_secs(expire_in),
            )),
        );
    }
}

pub struct SetPendingJob {
    pub key: Vec<u8>,
}

impl Message for SetPendingJob {
    type Result = ();
}

impl Handler<SetPendingJob> for StateActor {
    type Result = ();

    fn handle(&mut self, msg: SetPendingJob, _: &mut Context<Self>) {
        let expire_in = PROOF_TTL;
        self.states.insert(
            msg.key,
            RwLock::new((None, Instant::now() + Duration::from_secs(expire_in))),
        );
    }
}

pub struct GetProof {
    pub key: Vec<u8>,
}

impl Message for GetProof {
    type Result = Option<(Option<String>, Instant)>;
}

impl Handler<GetProof> for StateActor {
    type Result = Option<(Option<String>, Instant)>;

    fn handle(&mut self, msg: GetProof, _: &mut Context<Self>) -> Self::Result {
        self.states
            .get(&msg.key)
            .map(|inner| inner.read().unwrap().clone())
    }
}

struct CheckExpire;

impl Message for CheckExpire {
    type Result = ();
}

impl Handler<CheckExpire> for StateActor {
    type Result = ();

    fn handle(&mut self, _msg: CheckExpire, _ctx: &mut Context<Self>) {
        let now = Instant::now();
        // Remove expired states
        self.states.retain(|_, inner| {
            let expiry = inner.read().unwrap().1;
            expiry > now
        });
    }
}

impl Actor for StateActor {
    type Context = Context<Self>;

    // Schedule regular expiration checks when the actor starts
    fn started(&mut self, ctx: &mut Self::Context) {
        self.states.clear();

        // Check every PROOF_TTL seconds
        let interval = PROOF_TTL;
        ctx.run_interval(Duration::from_secs(interval), |_actor, ctx| {
            ctx.address().do_send(CheckExpire);
        });
    }
}
