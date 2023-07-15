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

// examples/03-routing.rs
// This node routes a message.

use colored::Colorize;
use hello_ockam::{print_title, Echoer, Hopper};
use ockam::{node, route, Context, Result};

/// From: <https://docs.ockam.io/reference/libraries/rust/routing#app-worker>
#[ockam::node]
async fn main(ctx: Context) -> Result<()> {
    print_title("Run a node w/ 'app', 'echoer' and 'h1' workers → send a message over a hop -> stop the node");

    // Create a node with default implementations
    let mut node = node(ctx);

    // Start a worker, of type Echoer, at address "echoer"
    node.start_worker("echoer", Echoer).await?;

    // Start a worker, of type Hopper, at address "h1"
    node.start_worker("h1", Hopper).await?;

    // Send a message to the worker at address "echoer",
    // via the worker at address "h1"
    let route = route!["h1", "echoer"];
    let route_msg = format!("{:?}", route);
    let msg = "Hello Ockam!".to_string();
    println!(
        "App Sending: '{0}', over route: '{1}'",
        msg.red(),
        route_msg.green()
    );
    node.send(route, msg).await?;

    // Wait to receive a reply and print it.
    let reply = node.receive::<String>().await?;
    println!("App Received: {}", reply); // should print "echo back: Hello Ockam!"

    // Stop all workers, stop the node, cleanup and return.
    node.stop().await
}
