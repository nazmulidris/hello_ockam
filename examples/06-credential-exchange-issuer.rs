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
use ockam::access_control::AllowAll;
use ockam::access_control::IdentityIdAccessControl;
use ockam::identity::CredentialsIssuer;
use ockam::identity::SecureChannelListenerOptions;
use ockam::TcpTransportExtension;
use ockam::{node, Context, Result, TcpListenerOptions};

/// From: <https://docs.ockam.io/reference/libraries/rust/credentials>
/// examples/06-credential-exchange-issuer.rs
/// Ockam enables you to define various pluggable Enrollment Protocols to decide who
/// should be issued credentials. For this example we'll assume that this list is known in
/// advance.
#[ockam::node]
async fn main(ctx: Context) -> Result<()> {
    print_title(
        "Create a node that runs a credential exchange issuer (creds are known in advance) on 5000 → wait for messages until stopped",
    );

    // Create a node with default implementations
    let node = node(ctx);

    let issuer_identity = "0180370b91c5d0aa4af34580a9ab4b8fb2a28351bed061525c96b4f07e75c0ee18000547c93239ba3d818ec26c9cdadd2a35cbdf1fa3b6d1a731e06164b1079fb7b8084f434b414d5f524b03012000000020236f79490d3f683e0c3bf458a7381c366c99a8f2b2ac406db1ef8c130111f12703010140b23fddceb11cea25602aa681b6ef6abda036722c27a6dee291f1d6b2234a127af21cc79de2252201f27e7e34e0bf5064adbf3d01eb355aff4bf5c90b8f1fd80a";
    let secret = "9278735d525efceef16bfd9143d3534759f3d388e460e6002134b9541e06489f";
    let issuer = node
        .import_private_identity(issuer_identity, secret)
        .await?;
    let output_msg = format!("🔒 issuer identifier {}", issuer.identifier());
    println!("{}", output_msg.on_bright_purple());

    // Tell the credential issuer about a set of public identifiers that are
    // known, in advance, to be members of the production cluster.
    let known_identifiers = vec![
        "Pe92f183eb4c324804ef4d62962dea94cf095a265d4d28500c34e1a4e0d5ef638".try_into()?,
        "Pada09e0f96e56580f6a0cb54f55ecbde6c973db6732e30dfb39b178760aed041".try_into()?,
    ];

    // Tell this credential issuer about the attributes to include in credentials
    // that will be issued to each of the above known_identifiers, after and only
    // if, they authenticate with their corresponding latest private key.
    //
    // Since this issuer knows that the above identifiers are for members of the
    // production cluster, it will issue a credential that attests to the attribute
    // set: [{cluster, production}] for all identifiers in the above list.
    //
    // For a different application this attested attribute set can be different and
    // distinct for each identifier, but for this example we'll keep things simple.
    let credential_issuer = CredentialsIssuer::new(
        node.identities(),
        issuer.identifier(),
        "trust_context".into(),
    )
    .await?;

    for identifier in known_identifiers.iter() {
        node.identities()
            .repository()
            .put_attribute_value(identifier, "cluster", "production")
            .await?;
    }

    let tcp_listener_options = TcpListenerOptions::new();
    let sc_listener_options = SecureChannelListenerOptions::new()
        .as_consumer(&tcp_listener_options.spawner_flow_control_id());
    let sc_listener_flow_control_id = sc_listener_options.spawner_flow_control_id();

    // Start a secure channel listener that only allows channels where the identity
    // at the other end of the channel can authenticate with the latest private key
    // corresponding to one of the above known public identifiers.
    node.create_secure_channel_listener(&issuer.identifier(), "secure", sc_listener_options)
        .await?;

    // Start a credential issuer worker that will only accept incoming requests from
    // authenticated secure channels with our known public identifiers.
    let allow_known = IdentityIdAccessControl::new(known_identifiers);
    node.flow_controls()
        .add_consumer("issuer", &sc_listener_flow_control_id);
    node.start_worker_with_access_control("issuer", credential_issuer, allow_known, AllowAll)
        .await?;

    // Initialize TCP Transport, create a TCP listener, and wait for connections.
    let tcp_transport = node.create_tcp_transport().await?;
    tcp_transport
        .listen("127.0.0.1:5000", tcp_listener_options)
        .await?;

    println!("{}", "🔒 issuer started".on_bright_purple());

    Ok(())
}

fn print_title(title: &str) {
    let padding = "=".repeat(title.len());
    println!("{}", padding.red().on_yellow());
    println!("{}", title.on_purple());
    println!("{}", padding.red().on_yellow());
}
