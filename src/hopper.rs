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
use ockam::{Any, Context, Result, Routed, Worker};

pub struct Hopper;

/// From: <https://docs.ockam.io/reference/libraries/rust/routing#hop-worker>
/// src/hop.rs
#[ockam::worker]
impl Worker for Hopper {
    type Context = Context;
    type Message = Any;

    /// This handle function takes any incoming message and forwards
    /// it to the next hop in it's onward route
    async fn handle_message(&mut self, ctx: &mut Context, msg: Routed<Any>) -> Result<()> {
        // Cast the msg to a Routed<String>
        let msg: Routed<String> = msg.cast()?;
        let output_msg = format!("🐇 Address: {}, Received: {}", ctx.address(), msg);
        println!("{}", output_msg.on_bright_blue());

        // Some type conversion
        let mut message = msg.into_local_message();
        let transport_message = message.transport_mut();

        // Remove my address from the onward_route
        let removed_address = transport_message.onward_route.step()?;

        let output_msg = format!(
            "\tonward_route -> remove address: {}, \n\treturn_route -> prepend address: {}",
            removed_address,
            ctx.address()
        );
        println!("{}", output_msg.on_bright_blue());

        // Insert my address at the beginning return_route
        transport_message
            .return_route
            .modify()
            .prepend(ctx.address());

        // Send the message on its onward_route
        ctx.forward(message).await
    }
}
