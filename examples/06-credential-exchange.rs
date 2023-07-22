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
use hello_ockam::Echoer;
use ockam::access_control::IdentityIdAccessControl;
use ockam::identity::CredentialsIssuer;
use ockam::identity::SecureChannelListenerOptions;
use ockam::TcpTransportExtension;
use ockam::{
    abac::AbacAccessControl,
    access_control::AllowAll,
    identity::{AuthorityService, CredentialsIssuerClient, SecureChannelOptions, TrustContext},
    route, AsyncTryClone, TcpConnectionOptions,
};
use ockam::{node, Context, Result, TcpListenerOptions};

/// From: <https://docs.ockam.io/reference/libraries/rust/credentials>
/// examples/06-credential-exchange-issuer.rs
/// Ockam enables you to define various pluggable Enrollment Protocols to decide who
/// should be issued credentials. For this example we'll assume that this list is known in
/// advance.
#[ockam::node]
async fn main(ctx: Context) -> Result<()> {
    let ctx_clone = ctx.async_try_clone().await?;
    let ctx_clone_2 = ctx.async_try_clone().await?;

    let mut node_issuer = create_issuer_node(ctx).await.unwrap();
    let mut node_server = create_server_node(ctx_clone).await.unwrap();

    create_client_node(ctx_clone_2).await.unwrap();

    node_server.stop().await.ok();
    node_issuer.stop().await.ok();

    Ok(())
}

async fn create_client_node(ctx: Context) -> Result<()> {
    print_title(
        "Create a node that is the client w/ identity known by issuer, connect to 5000 → stop",
    );

    // Create a node with default implementations
    let mut node = node(ctx);
    // Initialize the TCP Transport
    let tcp_transport = node.create_tcp_transport().await?;

    // Create an Identity representing the client
    // We preload the client vault with a change history and secret key corresponding to the identity identifier
    // Pe92f183eb4c324804ef4d62962dea94cf095a265d4d28500c34e1a4e0d5ef638
    // (derived from `change_history` & `secret` below)
    // which is an identifier known to the credential issuer, with some preset attributes
    // We're hard coding this specific identity because its public identifier is known
    // to the credential issuer as a member of the production cluster.
    let change_history = "01dcf392551f796ef1bcb368177e53f9a5875a962f67279259207d24a01e690721000547c93239ba3d818ec26c9cdadd2a35cbdf1fa3b6d1a731e06164b1079fb7b8084f434b414d5f524b03012000000020a0d205f09cab9a9467591fcee560429aab1215d8136e5c985a6b7dc729e6f08203010140b098463a727454c0e5292390d8f4cbd4dd0cae5db95606832f3d0a138936487e1da1489c40d8a0995fce71cc1948c6bcfd67186467cdd78eab7e95c080141505";
    let secret = "41b6873b20d95567bf958e6bab2808e9157720040882630b1bb37a72f4015cd2";
    let client = node.import_private_identity(change_history, secret).await?;

    // Connect with the credential issuer and authenticate using the latest private
    // key of this program's hardcoded identity.
    //
    // The credential issuer already knows the public identifier of this identity
    // as a member of the production cluster so it returns a signed credential
    // attesting to that knowledge.
    let issuer_connection = tcp_transport
        .connect("127.0.0.1:5000", TcpConnectionOptions::new())
        .await?;
    let issuer_channel = node
        .create_secure_channel(
            &client.identifier(),
            route![issuer_connection, "secure-issuer"],
            SecureChannelOptions::new(),
        )
        .await?;

    let issuer_client =
        CredentialsIssuerClient::new(route![issuer_channel, "issuer"], node.context()).await?;

    let credential = issuer_client.credential().await?;
    let output_msg = format!("❓ Retrieving credential from issuer:\n{credential}");
    println!("{}", output_msg.on_bright_blue());

    // Verify that the received credential has indeed be signed by the issuer.
    // The issuer identity must be provided out-of-band from a trusted source
    // and match the identity used to start the issuer node
    let issuer_identity = "0180370b91c5d0aa4af34580a9ab4b8fb2a28351bed061525c96b4f07e75c0ee18000547c93239ba3d818ec26c9cdadd2a35cbdf1fa3b6d1a731e06164b1079fb7b8084f434b414d5f524b03012000000020236f79490d3f683e0c3bf458a7381c366c99a8f2b2ac406db1ef8c130111f12703010140b23fddceb11cea25602aa681b6ef6abda036722c27a6dee291f1d6b2234a127af21cc79de2252201f27e7e34e0bf5064adbf3d01eb355aff4bf5c90b8f1fd80a";
    let issuer = node.import_identity_hex(issuer_identity).await?;
    node.credentials()
        .verify_credential(&client.identifier(), &[issuer.clone()], credential.clone())
        .await?;
    let output_msg = format!("Verify that the recieved credential is signed by the issuer");
    println!("{}", output_msg.on_bright_black());

    // Create a trust context that will be used to authenticate credential exchanges.
    // The trust context is needed to verify the credential.
    let trust_context = TrustContext::new(
        "trust_context_id".to_string(),
        Some(AuthorityService::new(
            node.identities().identities_reader(),
            node.credentials(),
            issuer.identifier(),
            None,
        )),
    );
    let output_msg = format!("Create a trust context (needed to verify the credential)");
    println!("{}", output_msg.on_bright_black());

    // Create a secure channel to the node that is running the Echoer service.
    let server_connection = tcp_transport
        .connect("127.0.0.1:4000", TcpConnectionOptions::new())
        .await?;
    let channel = node
        .create_secure_channel(
            &client.identifier(),
            route![server_connection, "secure-server"],
            SecureChannelOptions::new()
                .with_trust_context(trust_context)
                .with_credential(credential),
        )
        .await?;
    let output_msg = format!("Create a secure channel to echoers");
    println!("{}", output_msg.on_bright_black());

    // Send a message to the worker at address "echoer".
    // Wait to receive a reply and print it.
    let msg = "Hello Ockam!";
    let route = route![channel, "echoer"];
    let route_msg = format!("{:?}", route);
    let reply = node
        .send_and_receive::<String>(route, msg.to_string())
        .await?;
    let output_msg: String = format!(
        "App Sent: '{0}', via route: '{1}', Received: '{2}'",
        msg, route_msg, reply
    );
    println!("{}", output_msg.on_bright_black()); // Should print "👈 echo back:  Hello Ockam!");

    node.stop().await
}

async fn create_server_node(ctx: Context) -> Result<ockam::Node> {
    print_title(
        "Create a node that verifies credentials against the issuer, runs a tcp listener on 4000, secure channel listener, and echoer worker → wait for messages until stopped",
    );

    // Create a node with default implementations
    let node = node(ctx);
    // Initialize the TCP Transport
    let tcp = node.create_tcp_transport().await?;

    // Create an Identity representing the server
    // Load an identity corresponding to the following public identifier
    // `Pe92f183eb4c324804ef4d62962dea94cf095a265d4d28500c34e1a4e0d5ef638`
    // (derived from `change_history` & `secret` below).
    //
    // We're hard coding this specific identity because its public identifier is known
    // to the credential issuer as a member of the production cluster.
    let change_history = "01ed8a5b1303f975c1296c990d1bd3c1946cfef328de20531e3511ec5604ce0dd9000547c93239ba3d818ec26c9cdadd2a35cbdf1fa3b6d1a731e06164b1079fb7b8084f434b414d5f524b03012000000020e8c328bc0cc07a374762091d037e69c36fdd4d2e1a651abd4d43a1362d3f800503010140a349968063d7337d0c965969fa9c640824c01a6d37fe130d4ab963b0271b9d5bbf0923faa5e27f15359554f94f08676df01b99d997944e4feaf0caaa1189480e";
    let secret = "5b2b3f2abbd1787704d8f8b363529f8e2d8f423b6dd4b96a2c462e4f0e04ee18";
    let server = node.import_private_identity(change_history, secret).await?;

    // Connect with the credential issuer and authenticate using the latest private
    // key of this program's hardcoded identity.
    //
    // The credential issuer already knows the public identifier of this identity
    // as a member of the production cluster so it returns a signed credential
    // attesting to that knowledge.
    let issuer_connection = tcp
        .connect("127.0.0.1:5000", TcpConnectionOptions::new())
        .await?;
    let issuer_channel = node
        .create_secure_channel(
            &server.identifier(),
            route![issuer_connection, "secure-issuer"],
            SecureChannelOptions::new(),
        )
        .await?;

    let issuer_client =
        CredentialsIssuerClient::new(route![issuer_channel, "issuer"], node.context()).await?;
    let credential = issuer_client.credential().await?;
    let output_msg = format!("❓ Retrieving credential from issuer:\n{credential}");
    println!("{}", output_msg.on_bright_blue());

    // Verify that the received credential has indeed be signed by the issuer.
    // The issuer identity must be provided out-of-band from a trusted source
    // and match the identity used to start the issuer node
    let issuer_identity = "0180370b91c5d0aa4af34580a9ab4b8fb2a28351bed061525c96b4f07e75c0ee18000547c93239ba3d818ec26c9cdadd2a35cbdf1fa3b6d1a731e06164b1079fb7b8084f434b414d5f524b03012000000020236f79490d3f683e0c3bf458a7381c366c99a8f2b2ac406db1ef8c130111f12703010140b23fddceb11cea25602aa681b6ef6abda036722c27a6dee291f1d6b2234a127af21cc79de2252201f27e7e34e0bf5064adbf3d01eb355aff4bf5c90b8f1fd80a";
    let issuer = node.import_identity_hex(issuer_identity).await?;
    node.credentials()
        .verify_credential(&server.identifier(), &[issuer.clone()], credential.clone())
        .await?;
    let output_msg = format!("🔒✅ Credential verified as signed by the issuer:\n{credential}");
    println!("{}", output_msg.on_bright_blue());

    // Create a trust context that will be used to authenticate credential exchanges
    let trust_context = TrustContext::new(
        "trust_context_id".to_string(),
        Some(AuthorityService::new(
            node.identities().identities_reader(),
            node.credentials(),
            issuer.identifier(),
            None,
        )),
    );
    let output_msg = format!("🔒✅ Starting a trust context: \n{}", trust_context.id());
    println!("{}", output_msg.on_bright_blue());

    // Start an echoer worker that will only accept incoming requests from
    // identities that have authenticated credentials issued by the above credential
    // issuer. These credentials must also attest that requesting identity is
    // a member of the production cluster.
    let tcp_listener_options = TcpListenerOptions::new();
    let sc_listener_options = SecureChannelListenerOptions::new()
        .with_trust_context(trust_context)
        .with_credential(credential)
        .as_consumer(&tcp_listener_options.spawner_flow_control_id());

    node.flow_controls()
        .add_consumer("echoer", &sc_listener_options.spawner_flow_control_id());
    let allow_production = AbacAccessControl::create(node.repository(), "cluster", "production");
    node.start_worker_with_access_control("echoer", Echoer, allow_production, AllowAll)
        .await?;
    let output_msg = "🔒🪞 start echoer worker that only accepts requests from identities that have authenticated credentials issued by the above credential issuer, and have the right attributes";
    println!("{}", output_msg.on_bright_blue());

    // Start a secure channel listener that only allows channels with
    // authenticated identities.
    node.create_secure_channel_listener(&server.identifier(), "secure-server", sc_listener_options)
        .await?;
    let output_msg = "🔒🎙️ create secure channel listener that only allows channels w/ auth ids";
    println!("{}", output_msg.on_bright_blue());

    // Create a TCP listener and wait for incoming connections
    tcp.listen("127.0.0.1:4000", tcp_listener_options).await?;

    println!("{}", "🔒🖥️ server started on 4000".on_bright_blue());

    Ok(node)
}

async fn create_issuer_node(ctx: Context) -> Result<ockam::Node> {
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
    node.create_secure_channel_listener(&issuer.identifier(), "secure-issuer", sc_listener_options)
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

    Ok(node)
}

fn print_title(title: &str) {
    let padding = "=".repeat(title.len());
    println!("{}", padding.black().on_bright_white());
    println!("{}", title.black().on_bright_white());
    println!("{}", padding.black().on_bright_white());
}
