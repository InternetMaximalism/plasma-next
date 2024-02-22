use actix::Addr;
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use zkp::serialization::serialized_proof::SerializedProof;

use crate::state::{GetProof, SetPendingJob, SetProof, SnarkState, StateActor, PROOF_TTL};

#[post("/prove")]
async fn prove(state: web::Data<SnarkState>, proof: web::Json<SerializedProof>) -> impl Responder {
    let proof = proof.into_inner();
    let proof_hex = state.prove(proof);
    HttpResponse::Ok().json(proof_hex)
}

#[derive(Serialize, Deserialize)]
struct ProofJobResponseData {
    proof: Option<String>,
    elapsed_time: String,
}

#[post("/proof-job")]
async fn request_and_get_proof(
    state: web::Data<SnarkState>,
    addr: web::Data<Addr<StateActor>>,
    proof: web::Json<SerializedProof>,
) -> impl Responder {
    let proof = proof.into_inner();
    if proof.0.is_empty() {
        return HttpResponse::BadRequest().json("proof is empty");
    }

    #[cfg(feature = "debug")]
    {
        use sha2::Digest;

        let mut hasher = sha2::Sha256::new();
        hasher.update(proof.0.clone());
        let proof_digest = hasher.finalize();
        let proof_digest_hex = hex::encode(proof_digest);
        log::debug!("sha256 of proof: {proof_digest_hex}");
    }

    let proof_status = addr
        .send(GetProof {
            key: proof.0.clone(),
        })
        .await
        .expect("fail to get job");

    if let Some((proof_hex_opt, expiration)) = proof_status {
        let elapsed_time = Instant::now() + Duration::from_secs(PROOF_TTL) - expiration;
        if let Some(proof_hex) = proof_hex_opt {
            return HttpResponse::Ok().json(ProofJobResponseData {
                proof: Some(proof_hex),
                elapsed_time: elapsed_time.as_secs().to_string(),
            });
        }

        return HttpResponse::Ok().json(ProofJobResponseData {
            proof: None,
            elapsed_time: elapsed_time.as_secs().to_string(),
        });
    }

    // Job is not found.

    addr.send(SetPendingJob {
        key: proof.0.clone(),
    })
    .await
    .expect("fail to set job");
    actix_web::rt::spawn(async move {
        let proof_hex = state.prove(proof.clone());

        // debug assertion
        assert!(proof_hex.starts_with("0x"));

        addr.send(SetProof {
            key: proof.0,
            value: proof_hex.clone(),
        })
        .await
        .expect("fail to set proof");
    });
    println!("background job started.");

    HttpResponse::Ok().json(ProofJobResponseData {
        proof: None,
        elapsed_time: "0".to_string(),
    })
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().json("OK")
}

pub fn api_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(prove)
            .service(health)
            .service(request_and_get_proof),
    );
}

#[cfg(test)]
mod tests {

    use actix_http::Request;
    use actix_web::{
        dev::{Service, ServiceResponse},
        test, web, Error,
    };
    use zkp::serialization::serialized_proof::SerializedProof;

    use crate::{snark_processor::generate_proof_tuple_and_data, state::SnarkState};

    async fn post_helper<I, O>(
        app: &mut impl Service<Request, Response = ServiceResponse, Error = Error>,
        path: &str,
        input: I,
    ) -> O
    where
        I: serde::Serialize,
        O: serde::de::DeserializeOwned,
    {
        let req = test::TestRequest::post()
            .uri(path)
            .set_json(&input)
            .to_request();
        let resp = test::call_service(app, req).await;
        assert!(resp.status().is_success());
        let body = test::read_body(resp).await;
        serde_json::from_slice(&body).unwrap()
    }

    #[actix_web::test]
    async fn test_server_prove() {
        let state = SnarkState::new();
        let app_data = web::Data::new(state);
        let mut app = actix_web::test::init_service(
            actix_web::App::new()
                .app_data(app_data.clone())
                .configure(super::api_config),
        )
        .await;
        let (proof_tuple, data) = generate_proof_tuple_and_data();
        let serialized_proof = SerializedProof::from_proof(&data, &proof_tuple.0);
        let proof_hex: String = post_helper(&mut app, "/api/prove", serialized_proof).await;
        println!("{}", proof_hex);
    }
}
