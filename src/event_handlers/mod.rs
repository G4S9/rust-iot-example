use std::fs;

use redux::{Action, Store};
use rumqttc::{Event, Packet, Publish, QoS};
use serde::{Deserialize, Serialize};

use crate::{AppAction, AppError, AppState};

static CREATE_CERTIFICATE: &str = "$aws/certificates/create/json";
static CREATE_CERTIFICATE_ACCEPTED: &str = "$aws/certificates/create/json/accepted";
static CREATE_CERTIFICATE_REJECTED: &str = "$aws/certificates/create/json/rejected";
static PROVISION: &str = "$aws/provisioning-templates/iot_provisioning_template/provision/json";
static PROVISION_ACCEPTED: &str =
    "$aws/provisioning-templates/iot_provisioning_template/provision/json/accepted";
static PROVISION_REJECTED: &str =
    "$aws/provisioning-templates/iot_provisioning_template/provision/json/rejected";

static SUB_TOPICS: [&str; 4] = [
    CREATE_CERTIFICATE_ACCEPTED,
    CREATE_CERTIFICATE_REJECTED,
    PROVISION_ACCEPTED,
    PROVISION_REJECTED,
];

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateKeysAndCertificateResponse {
    certificate_id: String,
    certificate_ownership_token: String,
    certificate_pem: String,
    private_key: String,
}

#[derive(Serialize)]
struct RegisterThingRequestParameters {
    #[serde(rename = "SerialNumber")]
    serial_number: String,
    #[serde(rename = "AWS::IoT::Certificate::Id")]
    aws_iot_certificate_id: String,
}

#[derive(Serialize)]
struct RegisterThingRequest {
    #[serde(rename = "certificateOwnershipToken")]
    certificate_ownership_token: String,
    parameters: RegisterThingRequestParameters,
}

pub async fn handle_mqtt_event(
    store: &mut Store<AppState, AppAction>,
    e: Event,
) -> anyhow::Result<()> {
    let mqtt_client = store.select(|state| state.mqtt_client.clone());
    match e {
        Event::Incoming(Packet::ConnAck(_)) => {
            let provisioned = store.select(|state| state.provisioned);
            if !provisioned && !SUB_TOPICS.is_empty() {
                mqtt_client
                    .subscribe(SUB_TOPICS[0], QoS::AtLeastOnce)
                    .await?;
            }
        }
        Event::Incoming(Packet::SubAck(_)) => {
            let sub_count = store.select(|state| state.sub_count);
            store
                .dispatch(Action::Direct(AppAction::SetSubCount(sub_count)))
                .await?;
            if sub_count < SUB_TOPICS.len() {
                mqtt_client
                    .subscribe(SUB_TOPICS[sub_count], QoS::AtLeastOnce)
                    .await?;
            } else {
                mqtt_client
                    .publish(CREATE_CERTIFICATE, QoS::AtLeastOnce, false, "")
                    .await?;
            }
        }
        Event::Incoming(Packet::Publish(Publish { topic, payload, .. })) => {
            let payload_string = String::from_utf8(payload.to_vec())?;
            println!("Payload: {}", payload_string);
            if topic == CREATE_CERTIFICATE_ACCEPTED {
                let serial_number = store.select(|state| state.serial_number);
                let create_keys_and_certificate_response: CreateKeysAndCertificateResponse =
                    serde_json::from_slice(&payload)?;
                let register_thing_request_json = serde_json::to_string(&RegisterThingRequest {
                    certificate_ownership_token: create_keys_and_certificate_response
                        .certificate_ownership_token,
                    parameters: RegisterThingRequestParameters {
                        serial_number: String::from(serial_number),
                        aws_iot_certificate_id: create_keys_and_certificate_response.certificate_id,
                    },
                })?;
                fs::write(
                    "certificate.pem",
                    create_keys_and_certificate_response.certificate_pem,
                )?;
                fs::write(
                    "private_key.pem",
                    create_keys_and_certificate_response.private_key,
                )?;
                mqtt_client
                    .publish(
                        PROVISION,
                        QoS::AtLeastOnce,
                        false,
                        register_thing_request_json,
                    )
                    .await?;
            } else if topic == PROVISION_ACCEPTED {
                store
                    .dispatch(Action::Direct(AppAction::SetProvisioned(true)))
                    .await?;
                println!("PROVISIONED");
            } else if topic == CREATE_CERTIFICATE_REJECTED {
                store
                    .dispatch(Action::Direct(AppAction::SetError(
                        AppError::CertificateCreationError(payload_string),
                    )))
                    .await?;
            } else if topic == PROVISION_REJECTED {
                store
                    .dispatch(Action::Direct(AppAction::SetError(
                        AppError::ProvisioningError(payload_string),
                    )))
                    .await?;
            }
        }
        _ => {}
    };
    Ok(())
}
