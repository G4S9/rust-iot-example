use std::{env, fs, time::Duration};

use rand::Rng;
use rumqttc::{
    AsyncClient, Event, Key, MqttOptions, Packet, Publish, QoS, TlsConfiguration, Transport,
};
use serde::{Deserialize, Serialize};
use tokio::time;

#[derive(Serialize, Deserialize)]
struct Datum {
    value: f64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let serial_number = include_str!("res/serial_number.txt").trim();

    let (ca, client_cert, client_key) = (
        include_str!("res/AmazonRootCA1.pem").into(),
        fs::read("certificate.pem")?,
        fs::read("private_key.pem")?,
    );

    let mut mqtt_options = MqttOptions::new(
        serial_number,
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

    let (client, mut eventloop) = AsyncClient::new(mqtt_options, 10);

    client.subscribe(serial_number, QoS::AtLeastOnce).await?;

    tokio::spawn(async move {
        loop {
            let value = {
                let mut rng = rand::thread_rng();
                rng.gen_range(18.0..23.0)
            };
            if let Err(error) = client
                .publish(
                    serial_number,
                    QoS::AtLeastOnce,
                    false,
                    serde_json::to_string(&Datum { value }).unwrap(),
                )
                .await
            {
                eprintln!("Failed to publish data {:?}", error);
            };
            time::sleep(Duration::from_secs(5)).await;
        }
    });

    loop {
        match eventloop.poll().await {
            Ok(event) => {
                println!("Event = {:?}", event);
                if let Event::Incoming(Packet::Publish(Publish { payload, .. })) = event {
                    println!("{:?}", String::from_utf8(payload.to_vec()));
                }
            }
            Err(error) => {
                println!("Error in event loop: {:?}", error);
                return Err(error.into());
            }
        }
    }
}
