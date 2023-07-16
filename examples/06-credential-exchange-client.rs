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
use ockam::identity::{
    AuthorityService, CredentialsIssuerClient, SecureChannelOptions, TrustContext,
};
use ockam::TcpTransportExtension;
use ockam::{node, route, Context, Result, TcpConnectionOptions};

#[ockam::node]
async fn main(ctx: Context) -> Result<()> {
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
            route![issuer_connection, "secure"],
            SecureChannelOptions::new(),
        )
        .await?;

    let issuer_client =
        CredentialsIssuerClient::new(route![issuer_channel, "issuer"], node.context()).await?;
    let credential = issuer_client.credential().await?;
    let output_msg = format!("Credential:\n{credential}");
    println!("{}", output_msg.on_bright_black());

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
    let output_msg = format!("Create a trust context");
    println!("{}", output_msg.on_bright_black());

    // Create a secure channel to the node that is running the Echoer service.
    let server_connection = tcp_transport
        .connect("127.0.0.1:4000", TcpConnectionOptions::new())
        .await?;
    let channel = node
        .create_secure_channel(
            &client.identifier(),
            route![server_connection, "secure"],
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
    let output_msg = format!(
        "App Sent: '{0}', via route: '{1}', Received: '{2}'",
        msg, route_msg, reply
    );
    println!("{}", output_msg.on_bright_black()); // Should print "👈 echo back:  Hello Ockam!");

    node.stop().await
}

fn print_title(title: &str) {
    let padding = "=".repeat(title.len());
    println!("{}", padding.red().on_yellow());
    println!("{}", title.on_purple());
    println!("{}", padding.red().on_yellow());
}
