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

// src/echoer.rs

use colored::Colorize;
use ockam::{Context, Result, Routed, Worker};

pub struct Echoer;

/// From: <https://docs.ockam.io/reference/libraries/rust/nodes#workers>
/// src/echoer.rs
///
/// When a worker is started on a node, it is given one or more addresses. The node
/// maintains a mailbox for each address and whenever a message arrives for a specific
/// address it delivers that message to the corresponding registered worker.
///
/// Workers can handle messages from other workers running on the same or a different
/// node. In response to a message, an worker can: make local decisions, change its
/// internal state, create more workers, or send more messages to other workers running on
/// the same or a different node.
#[ockam::worker]
impl Worker for Echoer {
    type Context = Context;
    type Message = String;

    async fn handle_message(&mut self, ctx: &mut Context, msg: Routed<String>) -> Result<()> {
        // Echo the message body back on its return_route.
        let address_string = ctx.address().to_string();
        let msg_string = msg.as_body().to_string();
        let new_msg_string = format!("👈 echo back: {}", msg);

        let lines = [
            format!("📣 'echoer' worker → Address: {}", address_string.white()),
            format!("    Received: '{}'", msg_string.white()),
            format!("    Sent: '{}'", new_msg_string.white()),
        ];
        lines
            .iter()
            .for_each(|line| println!("{}", line.black().on_bright_magenta()));

        ctx.send(msg.return_route(), new_msg_string).await
    }
}
