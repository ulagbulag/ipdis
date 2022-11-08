mod args;

use clap::Parser;
use ipdis_common::Ipdis;
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
        args::Command::AddGuarantee { account } => {
            // sign as guarantor
            let data = client.sign_owned(*client.account_ref(), account)?;

            // external call
            client.add_guarantee(&data).await
        }
    }
}
