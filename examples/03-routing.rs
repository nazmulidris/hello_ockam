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

/// From: <https://docs.ockam.io/reference/libraries/rust/routing#app-worker>
/// examples/03-routing.rs
/// This node routes a message.
#[ockam::node]
async fn main(ctx: Context) -> Result<()> {
    print_title("Run a node w/ 'app', 'echoer' and 'hopper1' workers → send a message over a hop -> stop the node");

    println!("{}", HELP_TEXT.white().on_bright_black());

    // Create a node with default implementations
    let mut node = node(ctx);

    // Start a worker, of type Echoer, at address "echoer"
    node.start_worker("echoer", Echoer).await?;

    // Start a worker, of type Hopper, at address "hopper1"
    node.start_worker("hopper1", Hopper).await?;

    // Send a message to the worker at address "echoer",
    // via the worker at address "hopper1"
    let route = route!["hopper1", "echoer"];
    let route_msg = format!("{:?}", route);
    let msg = "Hello Ockam!".to_string();
    let output_msg = format!(
        "App Sending: '{0}', \nover route: '{1}'",
        msg.blue(),
        route_msg.red()
    );
    println!("{}", output_msg.on_bright_black());
    node.send(route, msg).await?;

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
┌──────────────────────┐
│  Node 1              │
├──────────────────────┤
│  ┌────────────────┐  │
│  │ Address:       │  │
│  │ 'app'          │  │
│  └─┬────────────▲─┘  │
│  ┌─▼────────────┴─┐  │
│  │ Address:       │  │
│  │ 'hopper1'      │  │
│  └─┬────────────▲─┘  │
│  ┌─▼────────────┴─┐  │
│  │ Address:       │  │
│  │ 'echoer'       │  │
│  └────────────────┘  │
└──────────────────────┘
"#;
