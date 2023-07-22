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
use hello_ockam::Echoer;
use ockam::{
    node, route, AsyncTryClone, Context, Result, TcpConnectionOptions, TcpListenerOptions,
    TcpTransportExtension,
};

/// From: <https://docs.ockam.io/reference/libraries/rust/routing#transport>
#[ockam::node]
async fn main(ctx: Context) -> Result<()> {
    println!("{}", HELP_TEXT.green());

    let ctx_clone = ctx.async_try_clone().await?;

    let mut node_responder = create_responder_node(ctx).await.unwrap();

    create_initiator_node(ctx_clone).await.unwrap();

    node_responder.stop().await.ok();

    println!("{}", "App finished, stopping responder node".red());

    Ok(())
}

/// examples/04-routing-over-transport-responder.rs
/// This node starts a tcp listener and an echoer worker.
/// It then runs forever waiting for messages.
async fn create_responder_node(ctx: Context) -> Result<ockam::Node> {
    print_title(
        "Create node_responder that runs tcp listener on 4000 and echoer worker → wait for messages until stopped",
    );

    // Create a node with default implementations
    let node = node(ctx);

    // Initialize the TCP Transport
    let tcp_transport = node.create_tcp_transport().await?;

    // Create an echoer worker
    node.start_worker("echoer", Echoer).await?;

    // Create a TCP listener and wait for incoming connections.
    let listener = tcp_transport
        .listen("127.0.0.1:4000", TcpListenerOptions::new())
        .await?;

    // Allow access to the Echoer via TCP connections from the TCP listener
    node.flow_controls()
        .add_consumer("echoer", listener.flow_control_id());

    Ok(node)
}

/// examples/04-routing-over-transport-initiator.rs
/// This node routes a message, to a worker on a different node, over the tcp transport.
async fn create_initiator_node(ctx: Context) -> Result<()> {
    print_title(
        "Create node_initiator that routes a message, over the TCP transport, to a worker on a different node → stop",
    );

    // Create a node with default implementations
    let mut node = node(ctx);

    // Initialize the TCP Transport.
    let tcp_transport = node.create_tcp_transport().await?;

    // Create a TCP connection to a different node.
    let connection_to_responder = tcp_transport
        .connect("localhost:4000", TcpConnectionOptions::new())
        .await?;

    // Send a message to the "echoer" worker on a different node, over a tcp transport.
    // Wait to receive a reply and print it.
    let msg = "Hello Ockam!";
    let route = route![connection_to_responder, "echoer"];
    let route_msg = format!("{:?}", route);
    let reply = node
        .send_and_receive::<String>(route, msg.to_string())
        .await?;

    let lines = [
        "node_initiator →".to_string(),
        format!("    sending: {}", msg.green()),
        format!("    over route: '{}'", route_msg.blue()),
        format!("    and received: '{}'", reply.purple()), // Should print "👈 echo back:  Hello Ockam!"
    ];
    lines
        .iter()
        .for_each(|line| println!("{}", line.black().on_white()));

    // Stop all workers, stop the node, cleanup and return.
    node.stop().await?;

    Ok(())
}

fn print_title(title: &str) {
    let padding = "=".repeat(title.len());
    println!("{}", padding.black().on_bright_white());
    println!("{}", title.black().on_bright_white());
    println!("{}", padding.black().on_bright_white());
}

#[rustfmt::skip]
const HELP_TEXT: &str =r#"
┌──────────────────────┐
│node_initiator        │
├──────────────────────┤
│ ┌──────────────────┐ │
│ │Address:          │ │  ┌──────────────────────┐
│ │'app'             │ │  │node_responder        │
│ └──┬────────────▲──┘ │  ├──────────────────────┤
│ ┌──▼────────────┴──┐ │  │ ┌──────────────────┐ │
│ │TCP transport     └─┼──┼─►TCP transport     │ │
│ │connect to 4000   ◄─┼──┼─┐listening on 4000 │ │
│ └──────────────────┘ │  │ └──┬────────────▲──┘ │
└──────────────────────┘  │ ┌──▼────────────┴──┐ │
                          │ │Address:          │ │
                          │ │'echoer'          │ │
                          │ └──────────────────┘ │
                          └──────────────────────┘
"#;
