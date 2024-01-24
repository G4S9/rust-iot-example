use std::{env, time::Duration};

use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
use aws_sdk_iot::{operation::describe_thing::DescribeThingError, Client as AwsIotSdkClient};
use rumqttc::{AsyncClient, Key, MqttOptions, TlsConfiguration, Transport};
use thiserror::Error;

use crate::event_handlers::handle_mqtt_event;
use redux::{Action, Store};

mod event_handlers;
mod reducer;

#[derive(Error, Debug, Clone)]
pub enum AppError {
    #[error("Error creating certificate: {0}")]
    CertificateCreationError(String),
    #[error("Error provisioning device: {0}")]
    ProvisioningError(String),
}

#[derive(Clone)]
pub struct AppState {
    serial_number: &'static str,
    mqtt_client: AsyncClient,
    provisioned: bool,
    sub_count: usize,
    error: Option<AppError>,
}

pub enum AppAction {
    SetSubCount(usize),
    SetProvisioned(bool),
    SetError(AppError),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let serial_number = include_str!("res/serial_number.txt").trim();

    let region_provider = RegionProviderChain::default_provider().or_else("eu-west-1");

    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;

    let aws_iot_sdk_client = AwsIotSdkClient::new(&config);

    let mut provisioned = true;

    if let Err(error) = aws_iot_sdk_client
        .describe_thing()
        .thing_name(serial_number)
        .send()
        .await
    {
        match error.as_service_error() {
            Some(DescribeThingError::ResourceNotFoundException(_)) => {
                provisioned = false;
            }
            _ => return Err(error.into()),
        }
    };

    if provisioned {
        println!("PROVISIONED");
        return Ok(());
    }

    let [ca, client_cert, client_key] = [
        include_str!("res/AmazonRootCA1.pem"),
        include_str!("res/claim_certificate.pem"),
        include_str!("res/claim_private_key.pem"),
    ]
    .map(|s| s.into());

    let mut mqtt_options = MqttOptions::new(
        "bootstrap",
        env::var("IOT_HOST")?,
        env::var("IOT_PORT")?.parse()?,
    );

    mqtt_options.set_keep_alive(Duration::from_secs(5));

    let transport = Transport::Tls(TlsConfiguration::Simple {
        ca,
        alpn: None,
        client_auth: Some((client_cert, Key::RSA(client_key))),
    });

    mqtt_options.set_transport(transport);

    let (mqtt_client, mut eventloop) = AsyncClient::new(mqtt_options, 10);

    let mut store = Store::new(
        AppState {
            sub_count: 0,
            serial_number,
            provisioned,
            mqtt_client,
            error: None,
        },
        reducer::reducer,
    );

    loop {
        match eventloop.poll().await {
            Ok(event) => {
                let formatted_event = format!("{:?}", event);
                println!("Event = {}", formatted_event);
                match store
                    .dispatch(Action::Thunk(Box::new(|store| {
                        Box::pin(handle_mqtt_event(store, event))
                    })))
                    .await
                {
                    Ok(_) => {
                        if store.select(|state| state.provisioned) {
                            break Ok(());
                        }
                        if let Some(app_error) = store.select(|state| state.error.clone()) {
                            break Err(app_error.into());
                        }
                    }
                    Err(error) => {
                        eprintln!(
                            "Failed to handle MQTT_EVENT: {}, err: {:?}",
                            formatted_event, error
                        );
                        break Err(error);
                    }
                }
            }
            Err(error) => {
                println!("Error in event loop: {:?}", error);
                break Err(error.into());
            }
        }
    }
}
