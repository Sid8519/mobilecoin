// Copyright (c) 2018-2022 The MobileCoin Foundation
#![deny(missing_docs)]

//! MobileCoin Fog View target
use clap::Parser;
use grpcio::{RpcStatus, RpcStatusCode};
use mc_attest_net::{Client, RaClient};
use mc_common::{logger::log, time::SystemTimeProvider};
use mc_fog_sql_recovery_db::SqlRecoveryDb;
use mc_fog_view_enclave::{SgxViewEnclave, ENCLAVE_FILE};
use mc_fog_view_server::{config::MobileAcctViewConfig, server::ViewServer};
use mc_util_grpc::AdminServer;
use std::{env, sync::Arc};

fn main() {
    mc_common::setup_panic_handler();
    let _sentry_guard = mc_common::sentry::init();
    let (logger, _global_logger_guard) =
        mc_common::logger::create_app_logger(mc_common::logger::o!());
    let config = MobileAcctViewConfig::parse();

    let recovery_db = SqlRecoveryDb::new_from_url(
        &std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable missing"),
        config.postgres_config.clone(),
        logger.clone(),
    )
    .expect("Failed connecting to database");

    let _tracer = mc_util_telemetry::setup_default_tracer_with_tags(
        env!("CARGO_PKG_NAME"),
        &[(
            "client_responser_id",
            config.client_responder_id.to_string(),
        )],
    )
    .expect("Failed setting telemetry tracer");

    let enclave_path = env::current_exe()
        .expect("Could not get the path of our executable")
        .with_file_name(ENCLAVE_FILE);
    log::info!(
        logger,
        "enclave path {}, responder ID {}",
        enclave_path.to_str().unwrap(),
        &config.client_responder_id
    );
    let sgx_enclave = SgxViewEnclave::new(
        enclave_path,
        config.client_responder_id.clone(),
        config.omap_capacity,
        logger.clone(),
    );

    let ias_client = Client::new(&config.ias_api_key).expect("Could not create IAS client");

    let mut server = ViewServer::new(
        config.clone(),
        sgx_enclave,
        recovery_db,
        ias_client,
        SystemTimeProvider::default(),
        logger.clone(),
    );
    server.start();

    let config2 = config.clone();
    let get_config_json = Arc::new(move || {
        serde_json::to_string(&config2)
            .map_err(|err| RpcStatus::with_message(RpcStatusCode::INTERNAL, format!("{:?}", err)))
    });
    let _admin_server = config.admin_listen_uri.as_ref().map(|admin_listen_uri| {
        AdminServer::start(
            None,
            admin_listen_uri,
            "Fog View".to_owned(),
            config.client_responder_id.to_string(),
            Some(get_config_json),
            logger,
        )
        .expect("Failed starting admin server")
    });

    loop {
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}
