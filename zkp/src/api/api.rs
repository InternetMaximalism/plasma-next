use actix_web::{
    get, post,
    web::{self, Data, Json},
    HttpResponse, Responder,
};
use log::error;

use crate::api::io::{AddInput, AppendToProofInput};

use crate::api::io::{SerializedBlockStatus, SyncBlockTreeInput, TickInput};

use super::{io::GenerateBlockInput, state::ServerState};

#[get("/get-status")]
pub async fn get_status(data: Data<ServerState>) -> impl Responder {
    let status = data.get_status();
    HttpResponse::Ok().json(status)
}

#[post("/generate-block")]
pub async fn generate_block(
    data: Data<ServerState>,
    req: Json<GenerateBlockInput>,
) -> impl Responder {
    let res = data.generate_block(req.into_inner());
    match res {
        Ok(block_info) => HttpResponse::Ok().json(block_info),
        Err(e) => {
            error!("generate-block error: {}", e.to_string());
            HttpResponse::InternalServerError().body(e.to_string())
        }
    }
}

#[post("/tick")]
pub async fn tick(data: Data<ServerState>, req: Json<TickInput>) -> impl Responder {
    let res = data.tick(req.into_inner());
    match res {
        Ok(block_status) => HttpResponse::Ok().json(block_status),
        Err(e) => {
            error!("tick error: {}", e.to_string());
            HttpResponse::InternalServerError().body(e.to_string())
        }
    }
}

#[post("/reset-block-tree")]
pub async fn reset_block_tree(data: Data<ServerState>) -> impl Responder {
    data.reset_block_tree();
    HttpResponse::Ok().json("reseted block tree")
}

#[post("/reset")]
pub async fn reset(data: Data<ServerState>) -> impl Responder {
    data.reset();
    HttpResponse::Ok().json("reseted")
}

#[post("/sync-block-tree")]
pub async fn sync_block_tree(
    data: Data<ServerState>,
    req: Json<SyncBlockTreeInput>,
) -> impl Responder {
    let res = data.sync_block_tree(req.into_inner());
    match res {
        Ok(()) => HttpResponse::Ok().json("synced block tree"),
        Err(e) => {
            error!("sync-block-tree error: {}", e.to_string());
            HttpResponse::InternalServerError().body(e.to_string())
        }
    }
}

#[get("/get-block-tree-status")]
pub async fn get_block_tree_status(data: Data<ServerState>) -> impl Responder {
    let block_tree_status = data.get_block_tree_status();
    HttpResponse::Ok().json(block_tree_status)
}

#[get("/get-snapshot-block-number")]
pub async fn get_snapshot_block_number(data: Data<ServerState>) -> impl Responder {
    let snapshot_block_number = data.get_snapshot_block_number();
    HttpResponse::Ok().json(snapshot_block_number)
}

#[post("/restore")]
pub async fn restore(data: Data<ServerState>, req: Json<SerializedBlockStatus>) -> impl Responder {
    let res = data.restore(req.into_inner());
    match res {
        Ok(()) => HttpResponse::Ok().json("restored"),
        Err(e) => {
            error!("restore error: {}", e.to_string());
            HttpResponse::InternalServerError().body(e.to_string())
        }
    }
}

#[post("/append-to-withdraw-proof")]
pub async fn append_to_withdraw_proof(
    data: Data<ServerState>,
    req: Json<AppendToProofInput>,
) -> impl Responder {
    let res = data.append_to_withdraw_proof(req.into_inner().into());
    match res {
        Ok(output) => HttpResponse::Ok().json(output),
        Err(e) => {
            error!("append-to-withdraw-proof error: {}", e.to_string());
            HttpResponse::InternalServerError().body(e.to_string())
        }
    }
}

#[post("/initialize")]
pub async fn initialize(data: Data<ServerState>) -> impl Responder {
    let snapshot_block_number = data.initialize();
    HttpResponse::Ok().json(snapshot_block_number)
}

#[post("/add")]
pub async fn add(data: Data<ServerState>, req: Json<AddInput>) -> impl Responder {
    let res = data.add(req.into_inner());
    match res {
        Ok(_) => HttpResponse::Ok().json("added"),
        Err(e) => {
            error!("add error: {}", e.to_string());
            HttpResponse::InternalServerError().body(e.to_string())
        }
    }
}

#[post("/finalize-and-wrap")]
pub async fn finalize_and_wrap(data: Data<ServerState>) -> impl Responder {
    let res = data.finalize_and_wrap();
    match res {
        Ok(finalize_output) => HttpResponse::Ok().json(finalize_output),
        Err(e) => {
            error!("finalize-and-wrap error: {}", e.to_string());
            HttpResponse::InternalServerError().body(e.to_string())
        }
    }
}

#[get("/health")]
pub async fn health() -> impl Responder {
    HttpResponse::Ok().body("OK!")
}

pub fn api_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(get_status)
            .service(generate_block)
            .service(tick)
            .service(reset_block_tree)
            .service(reset)
            .service(get_block_tree_status)
            .service(get_snapshot_block_number)
            .service(sync_block_tree)
            .service(restore)
            .service(append_to_withdraw_proof)
            .service(initialize)
            .service(add)
            .service(finalize_and_wrap)
            .service(health),
    );
}

#[cfg(test)]
mod tests {
    use actix_http::Request;
    use actix_web::{
        dev::{Service, ServiceResponse},
        test, web, App, Error,
    };
    use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};

    use crate::{
        api::{
            io::{
                AddInput, AppendToProofInput, AppendToProofOutput, FinalizeOutput,
                GenerateBlockInput, SerializedBlockInfo, SerializedBlockStatus, TickInput,
            },
            state::ServerState,
        },
        common::{address::Address, asset::Assets},
        random::transfers::generate_random_transfers,
    };

    use super::api_config;

    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

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
        assert!(resp.status().is_success(), "response: {:?}", resp);
        let body = test::read_body(resp).await;
        serde_json::from_slice(&body).unwrap()
    }

    #[actix_web::test]
    async fn test_server_to_finalize() {
        let status = ServerState::new();
        let app_data = web::Data::new(status);
        let mut app =
            test::init_service(App::new().app_data(app_data.clone()).configure(api_config)).await;
        // generate block
        let block_info: SerializedBlockInfo = post_helper(
            &mut app,
            "/api/generate-block",
            GenerateBlockInput {
                transfers: vec![],
                deposit: Assets::default(),
            },
        )
        .await;
        // tick
        let _block_status: SerializedBlockStatus = post_helper(
            &mut app,
            "/api/tick",
            TickInput {
                spent_proof: block_info.spent_proof,
            },
        )
        .await;
        // initialize
        let _snapshot_block_number: String = post_helper(&mut app, "/api/initialize", ()).await;
        // finalize
        let _finalize_output: FinalizeOutput =
            post_helper(&mut app, "/api/finalize-and-wrap", ()).await;
    }

    #[actix_web::test]
    async fn test_server_settlement_wrap() {
        let does_print = true;

        let status = ServerState::new();
        let app_data = web::Data::new(status);
        let mut app =
            test::init_service(App::new().app_data(app_data.clone()).configure(api_config)).await;
        let mut rng = rand::thread_rng();
        let recipients = vec![Address::rand(&mut rng)];
        let transfers_vec = generate_random_transfers::<F, _>(&mut rng, 1, 1, &recipients);
        let mut deposits = vec![Assets::rand_full(&mut rng)];
        deposits.resize(transfers_vec.len(), Assets::default());
        let mut withdraws = vec![];
        for (transfers, deposit) in transfers_vec.iter().zip(deposits.iter()) {
            let block_info: SerializedBlockInfo = post_helper(
                &mut app,
                "/api/generate-block",
                GenerateBlockInput {
                    transfers: transfers.to_vec(),
                    deposit: deposit.clone(),
                },
            )
            .await;
            let _block_status: SerializedBlockStatus = post_helper(
                &mut app,
                "/api/tick",
                TickInput {
                    spent_proof: block_info.spent_proof,
                },
            )
            .await;
            withdraws.extend(block_info.transfer_info);
        }
        let _snapshot_block_number: String = post_helper(&mut app, "/api/initialize", ()).await;
        let input = AppendToProofInput {
            transfer_info: withdraws.clone(),
            withdraw_proof: None,
        };
        let output: AppendToProofOutput =
            post_helper(&mut app, "/api/append-to-withdraw-proof", input).await;
        let add_input = AddInput {
            withdraw_proof: output.withdraw_proof,
            evidence_transfer_info: withdraws[0].clone(),
        };
        let _: String = post_helper(&mut app, "/api/add", add_input).await;
        let finalize_res: FinalizeOutput =
            post_helper(&mut app, "/api/finalize-and-wrap", ()).await;
        if does_print {
            let wrap_public_inputs = finalize_res.wrap_public_inputs.unwrap();
            let serialized_proof =
                serde_json::to_string(&finalize_res.wrap_proof.unwrap()).unwrap();
            println!("wrap_public_inputs: {}", wrap_public_inputs);
            println!("proof: {}", serialized_proof);
        }
    }
}
