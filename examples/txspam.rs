// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example txspam --release
use iota::{Client, MessageId, Payload, Seed, UTXOInput};
use tokio::time::sleep;
extern crate dotenv;
use dotenv::dotenv;
use std::{env, time::Duration};

/// In this example, we spam transactions
/// Send 10 Mi from the faucet to the first address before you run this

#[tokio::main]
async fn main() {
    let iota = Client::builder() // Crate a client instance builder
        .with_node("http://api.lb-0.testnet.chrysalis2.com") // Insert the node here
        .unwrap()
        .finish()
        .unwrap();

    dotenv().ok();
    let seed = Seed::from_ed25519_bytes(&hex::decode(env::var("seed").unwrap()).unwrap()).unwrap();

    // split funds to own addresses
    let addresses = iota
        .find_addresses(&seed)
        .with_account_index(0)
        .with_range(0..10)
        .finish()
        .unwrap();

    let mut message_builder = iota.send().with_seed(&seed);
    for address in &addresses {
        message_builder = message_builder.with_output(address, 1_000_000).unwrap();
    }
    let message = message_builder.finish().await.unwrap();

    println!(
        "First transaction sent: https://explorer.iota.org/chrysalis/message/{}",
        message.id().0
    );
    reattach_promote_until_confirmed(message.id().0, &iota).await;
    // At this point we have 10 Mi on 10 addresses and will just send it to their addresses again

    let mut initial_outputs = Vec::new();
    if let Some(Payload::Transaction(tx)) = message.payload() {
        for (index, _output) in tx.essence().outputs().iter().enumerate() {
            initial_outputs.push(UTXOInput::new(tx.id(), index as u16).unwrap());
        }
    }

    for (index, address) in addresses.iter().enumerate() {
        let message = iota
            .send()
            .with_seed(&seed)
            .with_input(initial_outputs[index].clone())
            .with_output(address, 1_000_000)
            .unwrap()
            .finish()
            .await
            .unwrap();
        println!(
            "Tx sent: https://explorer.iota.org/chrysalis/message/{}",
            message.id().0
        );
    }
}

async fn reattach_promote_until_confirmed(message_id: MessageId, iota: &Client) {
    while let Ok(metadata) = iota.get_message().metadata(&message_id).await {
        if let Some(state) = metadata.ledger_inclusion_state {
            println!("Leder inclusion state: {:?}", state);
            break;
        } else if let Ok(msg_id) = iota.reattach(&message_id).await {
            println!("Reattached or promoted {}", msg_id.0);
        }
        sleep(Duration::from_secs(5)).await;
    }
}