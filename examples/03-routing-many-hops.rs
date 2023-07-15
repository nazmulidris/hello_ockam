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

// examples/03-routing-many-hops.rs
// This node routes a message through many hops.

use colored::Colorize;
use hello_ockam::{print_title, Echoer, Hopper};
use ockam::{node, route, Context, Result};

/// More info: <https://docs.ockam.io/reference/libraries/rust/routing#routing-over-many-hops>
#[ockam::node]
async fn main(ctx: Context) -> Result<()> {
    print_title("Run a node w/ 'app', 'echoer' and 'h1', 'h2', 'h3' workers → send a message over 3 hops -> stop the node");

    // Create a node with default implementations
    let mut node = node(ctx);

    // Start an Echoer worker at address "echoer"
    node.start_worker("echoer", Echoer).await?;

    // Start 3 hop workers at addresses "h1", "h2" and "h3".
    node.start_worker("h1", Hopper).await?;
    node.start_worker("h2", Hopper).await?;
    node.start_worker("h3", Hopper).await?;

    // Send a message to the echoer worker via the "h1", "h2", and "h3" workers
    let route = route!["h1", "h2", "h3", "echoer"];
    let route_msg = format!("{:?}", route);
    let msg = "Hello Ockam!".to_string();
    println!(
        "App Sending: '{0}', over route: '{1}'",
        msg.red(),
        route_msg.green()
    );
    node.send(route, "Hello Ockam!".to_string()).await?;

    // Wait to receive a reply and print it.
    let reply = node.receive::<String>().await?;
    println!("App Received: {}", reply); // should print "Hello Ockam!"

    // Stop all workers, stop the node, cleanup and return.
    node.stop().await
}
