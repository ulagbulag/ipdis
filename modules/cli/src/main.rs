mod args;

use std::env;

use clap::Parser;
use ipdis_common::{Ipdis, KIND};
use ipiis_api::{client::IpiisClient, common::Ipiis};
use ipis::{core::anyhow::Result, env::Infer, tokio};

#[tokio::main]
async fn main() -> Result<()> {
    // init logger
    ::ipis::logger::init_once();

    // parse the command-line arguments
    let args = args::Args::parse();

    // init client
    let client = IpiisClient::try_infer().await?;

    // execute a command
    match args.command {
        args::Command::AddGuarantee { guarantor } => {
            // init client
            let server = {
                env::set_var("ipis_account_me", guarantor.to_string());
                env::set_var(
                    "ipiis_router_db",
                    format!(
                        "/tmp/ipdis-modules-cli-ipiis-router-db-{}",
                        guarantor.account_ref().to_string(),
                    ),
                );
                IpiisClient::try_infer().await?
            };

            // add the primary address
            {
                server
                    .set_address(
                        KIND.as_ref(),
                        server.account_ref(),
                        &client
                            .get_address(KIND.as_ref(), server.account_ref())
                            .await?,
                    )
                    .await?;
                server
                    .set_account_primary(KIND.as_ref(), server.account_ref())
                    .await?;
            }

            // sign as guarantor
            let guarantee = server.sign_as_guarantor(
                client.sign_owned(*server.account_ref(), *client.account_ref())?,
            )?;

            // external call
            server.add_guarantee_unchecked(&guarantee).await
        }
    }
}
