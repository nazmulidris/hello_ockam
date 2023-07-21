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
use ockam::{node, Context, Result};

/// From: <https://docs.ockam.io/reference/libraries/rust/nodes>
/// examples/01-node.rs
/// This program creates and then immediately stops a node.
#[ockam::node]
async fn main(ctx: Context) -> Result<()> {
    print_title("Run a node & stop it right away");

    println!("{}", HELP_TEXT.white().on_bright_black());

    // Create a node with default implementations
    let mut node = node(ctx);

    // Stop the node as soon as it starts.
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
│  └────────────────┘  │
└──────────────────────┘
"#;
