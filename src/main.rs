use amiquip::{
    AmqpProperties, Connection, ConsumerMessage, ConsumerOptions, Exchange, Publish,
    QueueDeclareOptions, Result,
};
use log::{error, info, trace};

mod message;
mod sqip;

use crate::message::{SqipCreateMessage, SqipDoneMessage};
use crate::sqip::generate_sqip;

const SQIP_CREATE_QUEUE: &str = "sqip.create";
const SQIP_DONE_QUEUE: &str = "sqip.done";

#[tokio::main]
async fn main() -> Result<()> {
    let _ = kankyo::init();
    env_logger::init();

    let addr = std::env::var("AMQP_ADDR").expect("No AMQP_ADDR found");
    let is_insecure = std::env::var("INSECURE_AMQP_ADDR").or_else(|_| Ok(String::new()))?;

    let mut connection;
    if is_insecure.is_empty() {
        connection = Connection::open(&addr)?;
    } else {
        connection = Connection::insecure_open(&addr)?;
    }

    let channel = connection.open_channel(None)?;
    let exchange = Exchange::direct(&channel);
    let consumer_queue = channel.queue_declare(
        SQIP_CREATE_QUEUE,
        QueueDeclareOptions {
            durable: true,
            ..QueueDeclareOptions::default()
        },
    )?;
    // Set QOS to only send us 1 message at a time.
    channel.qos(0, 1, false)?;

    let consumer = consumer_queue.consume(ConsumerOptions::default())?;
    info!("Waiting for messages: {}", SQIP_CREATE_QUEUE);

    for (i, message) in consumer.receiver().iter().enumerate() {
        match message {
            ConsumerMessage::Delivery(delivery) => {
                let message: SqipCreateMessage =
                    serde_json::from_slice(&delivery.body).expect("Failed to obtain JSON");
                trace!("({:>3}) Received [{:?}]", i, message);

                let done_message: SqipDoneMessage = generate_sqip(&message).await.unwrap();


                exchange.publish(Publish::with_properties(
                    &serde_json::to_vec(&done_message).expect("Expect message to serialize"),
                    SQIP_DONE_QUEUE,
                    AmqpProperties::default(),
                ))?;
                consumer.ack(delivery)?;
            }
            other => {
                error!("Consumer ended: {:?}", other);
                break;
            }
        }
    }

    connection.close()
}
