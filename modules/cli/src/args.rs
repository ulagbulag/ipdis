use clap::{Parser, Subcommand};
use ipis::core::account::AccountRef;

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    AddGuarantee {
        /// Account of the target server
        #[clap(long, env = "ipiis_client_account")]
        account: AccountRef,
    },
}
