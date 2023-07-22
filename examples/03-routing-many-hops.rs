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
use hello_ockam::{Echoer, Hopper};
use ockam::{node, route, Context, Result};

/// From: <https://docs.ockam.io/reference/libraries/rust/routing#routing-over-many-hops>
/// examples/03-routing-many-hops.rs
/// This node routes a message through many hops.
#[ockam::node]
async fn main(ctx: Context) -> Result<()> {
    print_title("Run a node w/ 'app', 'echoer' and 'hopper1', 'hopper2', 'hopper3' workers → send a message over 3 hops -> stop the node");

    println!("{}", HELP_TEXT.green());

    // Create a node with default implementations
    let mut node = node(ctx);

    // Start an Echoer worker at address "echoer"
    node.start_worker("echoer", Echoer).await?;

    // Start 3 hop workers at addresses "hopper1", "hopper2" and "hopper3".
    node.start_worker("hopper1", Hopper).await?;
    node.start_worker("hopper2", Hopper).await?;
    node.start_worker("hopper3", Hopper).await?;

    // Send a message to the echoer worker via the "hopper1", "hopper2", and "hopper3" workers
    let route = route!["hopper1", "hopper2", "hopper3", "echoer"];
    let route_msg = format!("{:?}", route);
    let msg = "Hello Ockam!".to_string();
    let output_msg = format!(
        "App Sending: '{0}', over route: '{1}'",
        msg.red(),
        route_msg.green()
    );
    println!("{}", output_msg.on_bright_black());
    node.send(route, "Hello Ockam!".to_string()).await?;

    // Wait to receive a reply and print it.
    let reply = node.receive::<String>().await?;
    let output_msg = format!("App Received: '{}'", reply);
    println!("{}", output_msg.on_bright_black()); // Should print "👈 echo back:  Hello Ockam!"

    // Stop all workers, stop the node, cleanup and return.
    node.stop().await
}

fn print_title(title: &str) {
    let padding = "=".repeat(title.len());
    println!("{}", padding.black().on_bright_white());
    println!("{}", title.black().on_bright_white());
    println!("{}", padding.black().on_bright_white());
}

#[rustfmt::skip]
const HELP_TEXT: &str =r#"
┌───────────────────────┐
│  Node 1               │
├───────────────────────┤
│  ┌────────────────┐   │
│  │ Address:       │   │
│  │ 'app'          │   │
│  └─┬────────────▲─┘   │
│  ┌─▼────────────┴─┐   │
│  │ Address:       │   │
│  │ 'hopper1..3'   │x3 │
│  └─┬────────────▲─┘   │
│  ┌─▼────────────┴─┐   │
│  │ Address:       │   │
│  │ 'echoer'       │   │
│  └────────────────┘   │
└───────────────────────┘
"#;
