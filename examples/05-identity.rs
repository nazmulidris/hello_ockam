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

use ockam::node;
use ockam::{Context, Result};

#[ockam::node]
async fn main(ctx: Context) -> Result<()> {
    // Create default node to safely store secret keys for Alice
    let mut node = node(ctx);

    // Create an Identity to represent Alice.
    let alice = node.create_identity().await?;

    println!("Identity identifier for Alice: {:?}", alice);

    // Stop the node.
    node.stop().await
}
