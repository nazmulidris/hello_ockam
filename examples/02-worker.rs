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

// examples/02-worker.rs
// This node creates a worker, sends it a message, and receives a reply.

use hello_ockam::Echoer;
use ockam::{node, Context, Result};

/// "app" worker - `main()` is a worker w/ the address of "app" on this node.
#[ockam::node]
async fn main(ctx: Context) -> Result<()> {
    // Create a node with default implementations
    let mut node = node(ctx);

    // Start a worker, of type Echoer, at address "echoer"
    node.start_worker("echoer", Echoer).await?;

    // Send a message to the worker at address "echoer".
    node.send("echoer", "Hello Ockam!".to_string()).await?;

    // Wait to receive a reply and print it.
    let reply = node.receive::<String>().await?;
    println!("App Received: {}", reply); // should print "echo back: Hello Ockam!"

    // Stop all workers, stop the node, cleanup and return.
    node.stop().await
}
