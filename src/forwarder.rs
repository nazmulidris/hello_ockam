/*i
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
use ockam::{Address, Any, Context, LocalMessage, Result, Routed, Worker};

pub struct Forwarder {
    pub address: Address,
}

/// From: <https://docs.ockam.io/reference/libraries/rust/routing#routing-over-two-transport-hops>r
/// src/forwarder.rs
#[ockam::worker]
impl Worker for Forwarder {
    type Context = Context;
    type Message = Any;

    /// This handle function takes any incoming message and forwards
    /// it to the next hop in it's onward route
    async fn handle_message(&mut self, ctx: &mut Context, msg: Routed<Any>) -> Result<()> {
        let output_msg = format!(
            "👉 Address: {}, Received: {}",
            format!("{}", ctx.address()).white(),
            format!("{}", msg).white()
        );
        println!("{}", output_msg.on_bright_blue());

        // Some type conversion
        let mut transport_message = msg.into_local_message().into_transport_message();

        transport_message
            .onward_route
            .modify()
            .pop_front() // Remove my address from the onward_route
            .prepend(self.address.clone()); // Prepend predefined address to the onward_route

        let prev_hop = transport_message.return_route.next()?.clone();

        // Wipe all local info (e.g. transport types)
        let message = LocalMessage::new(transport_message, vec![]);

        if let Some(info) = ctx
            .flow_controls()
            .find_flow_control_with_producer_address(&self.address)
        {
            ctx.flow_controls()
                .add_consumer(prev_hop.clone(), info.flow_control_id());
        }

        if let Some(info) = ctx
            .flow_controls()
            .find_flow_control_with_producer_address(&prev_hop)
        {
            ctx.flow_controls()
                .add_consumer(self.address.clone(), info.flow_control_id());
        }

        // Send the message on its onward_route
        ctx.forward(message).await
    }
}
