/*
 *   Copyright (c) 2023 Nazmul Idris
 *   All rights reserved.
 *
 *   Licensed under the Apache License, Version 2.0 (the "License");
 *   you may not use this file except in compliance with the License.
 *   You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 *   Unless required by applicable law or agreed to in writing, software
 *   distributed under the License is distributed on an "AS IS" BASIS,
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *   See the License for the specific language governing permissions and
 *   limitations under the License.
 */

use colored::Colorize;
use hello_ockam::{Echoer, Forwarder};
use ockam::{
    identity::{SecureChannelListenerOptions, SecureChannelOptions},
    route, TcpConnectionOptions,
};
use ockam::{node, AsyncTryClone, Context, Result, TcpListenerOptions, TcpTransportExtension};
use tokio::spawn;

/// From: <https://docs.ockam.io/reference/libraries/rust/secure-channels>
#[ockam::node]
async fn main(ctx: Context) -> Result<()> {
    let ctx_clone = ctx.async_try_clone().await?;
    let ctx_clone_2 = ctx.async_try_clone().await?;

    let mut node_responder = spawn(async move { create_responder_node(ctx).await.unwrap() })
        .await
        .unwrap();

    let mut node_middle = spawn(async move { create_middle_node(ctx_clone).await.unwrap() })
        .await
        .unwrap();

    spawn(async move {
        create_initiator_node(ctx_clone_2).await.unwrap();
    })
    .await
    .ok();

    node_responder.stop().await.ok();
    node_middle.stop().await.ok();

    println!(
        "{}",
        "App finished, stopping responder & middle nodes".red()
    );

    Ok(())
}

/// examples/05-secure-channel-over-two-transport-hops-responder.rs
/// This node starts a tcp listener on 4000, a secure channel listener, and an echoer
/// worker. It then runs forever waiting for messages.
async fn create_responder_node(ctx: Context) -> Result<ockam::Node> {
    print_title(
        "Create a node that runs tcp listener on 4000, a secure channel listener (for `bob`) to an echoer worker → wait for messages until stopped",
    );

    // Create a node with default implementations
    let node = node(ctx);

    // Initialize the TCP Transport.
    let tcp_transport = node.create_tcp_transport().await?;

    node.start_worker("echoer", Echoer).await?;

    // Create an identity `bob`.
    let id_bob = node.create_identity().await?;

    // Create a TCP listener and wait for incoming connections.
    let listener = tcp_transport
        .listen("127.0.0.1:4000", TcpListenerOptions::new())
        .await?;

    // Create a secure channel listener for `bob` that will wait for requests to
    // initiate an Authenticated Key Exchange.
    let secure_channel_listener = node
        .create_secure_channel_listener(
            &id_bob,
            "bob_listener",
            SecureChannelListenerOptions::new().as_consumer(listener.flow_control_id()),
        )
        .await?;

    // Allow access to the Echoer via Secure Channels
    node.flow_controls()
        .add_consumer("echoer", secure_channel_listener.flow_control_id());

    Ok(node)
}

/// examples/05-secure-channel-over-two-transport-hops-middle.rs
/// This node creates a tcp connection to a node at 127.0.0.1:4000.
/// Starts a forwarder worker to forward messages to 127.0.0.1:4000.
/// Starts a tcp listener at 127.0.0.1:3000.
/// It then runs forever waiting to route messages.
async fn create_middle_node(ctx: Context) -> Result<ockam::Node> {
    print_title("Create a middle (forwarder) node that listens for TCP on 3000 and forwards to 4000 (no secure channel) → wait for messages until stopped");

    // Create a node with default implementations
    let node = node(ctx);

    // Initialize the TCP Transport
    let tcp_transport = node.create_tcp_transport().await?;

    // Create a TCP connection to `bob`.
    let tcp_connection_to_bob = tcp_transport
        .connect("127.0.0.1:4000", TcpConnectionOptions::new())
        .await?;

    // Start a Forwarder to forward messages to `bob` using the TCP connection.
    node.start_worker(
        "forward_to_bob",
        Forwarder {
            address: tcp_connection_to_bob.into(),
        },
    )
    .await?;

    // Create a TCP listener and wait for incoming connections.
    let listener = tcp_transport
        .listen("127.0.0.1:3000", TcpListenerOptions::new())
        .await?;

    node.flow_controls()
        .add_consumer("forward_to_bob", listener.flow_control_id());

    // Don't call node.stop() here so this node runs forever.
    Ok(node)
}

/// examples/05-secure-channel-over-two-transport-hops-initiator.rs
/// This node creates an end-to-end encrypted secure channel over two tcp transport hops.
/// It then routes a message, to a worker on a different node, through this encrypted channel.
async fn create_initiator_node(ctx: Context) -> Result<()> {
    print_title(
        "Create a node that creates an end-to-end encrypted secure channel (from `alice`), over two TCP transport hops, and routes a message (to `bob`), to a worker on a different node → stop",
    );

    // Create a node with default implementations
    let mut node = node(ctx);

    // Create an Identity to represent `alice`.
    let id_alice = node.create_identity().await?;

    // Create a TCP connection to the middle node.
    let tcp_transport = node.create_tcp_transport().await?;
    let connection_to_middle_node = tcp_transport
        .connect("localhost:3000", TcpConnectionOptions::new())
        .await?;

    // Connect to a secure channel listener and perform a handshake.
    let channel_route = route![connection_to_middle_node, "forward_to_bob", "bob_listener"];
    let channel_route_msg = format!("{:?}", channel_route);
    let channel = node
        .create_secure_channel(&id_alice, channel_route, SecureChannelOptions::new())
        .await?;
    println!(
        "Connected to secure channel listener from 'alice' after performing handshake: {}",
        channel_route_msg.green()
    );

    // Send a message to the echoer worker via the channel.
    // Wait to receive a reply and print it.
    let route = route![channel, "echoer"];
    let route_msg = format!("{:?}", route);
    let msg = "Hello Ockam!";
    let reply = node
        .send_and_receive::<String>(route, msg.to_string())
        .await?;
    let output_msg = format!(
        "App Sending: '{0}', \nover route: '{1}', \nand received: '{2}'",
        msg.red(),
        route_msg.green(),
        reply.yellow() // Should print "👈 echo back:  Hello Ockam!");
    );
    println!("{}", output_msg.on_bright_black());

    // Stop all workers, stop the node, cleanup and return.
    node.stop().await
}

fn print_title(title: &str) {
    let padding = "=".repeat(title.len());
    println!("{}", padding.red().on_yellow());
    println!("{}", title.on_purple());
    println!("{}", padding.red().on_yellow());
}

