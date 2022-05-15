use ipdis_api::{client::IpdisClient, server::IpdisServer};
use ipdis_common::Ipdis;
use ipiis_api::{client::IpiisClient, common::Ipiis, server::IpiisServer};
use ipis::{
    core::{
        anyhow::Result,
        value::{text::Text, word::Word},
    },
    env::Infer,
    tokio,
};

#[tokio::main]
async fn main() -> Result<()> {
    // deploy a server
    let server = IpdisServer::genesis(5001)?;
    let server_account = {
        let server: &IpiisServer = server.as_ref();
        let account = server.account_me();

        // register the environment variables
        ::std::env::set_var("ipis_account_me", account.to_string());

        account.account_ref()
    };
    tokio::spawn(async move { server.run().await });

    // create a guarantor client
    let client_guarantor = IpdisClient::infer();

    // create a client
    let client = IpiisClient::genesis(Some(server_account))?;
    let client_account = client.account_me().account_ref();
    client.add_address(server_account, "127.0.0.1:5001".parse()?)?;

    // register the client as guarantee
    {
        // sign as guarantor
        let guarantee = client.sign(server_account, client_account)?;

        client_guarantor.add_guarantee_unchecked(&guarantee).await?;
    };

    // create a sample word to be stored
    let kind = "ipdis-api-postgres-test";
    let word = Word {
        kind: kind.to_string(),
        text: Text::with_en_us("hello world"),
    };

    // make it hash
    let word = word.into();

    // put the word in IPDIS (* 3 times)
    let count = 3usize;
    for _ in 0..count {
        // sign as guarantee
        let word = client.sign(server_account, word)?;

        // put the word in IPDIS
        client.put_idf_log_unchecked(&word).await?;
    }

    // get the word log
    let word_from_ipdis = client.get_idf_log_unchecked(None, &word).await?.unwrap();
    assert_eq!(&word_from_ipdis.data.data.data, &word,);

    // get the word counts
    let count_from_ipdis = client.get_idf_count_unchecked(&word).await?;
    assert_eq!(count_from_ipdis, count);

    // get the word counts of the account
    let count_from_ipdis = client
        .get_idf_count_with_guarantee_unchecked(&client_account, &word)
        .await?;
    assert_eq!(count_from_ipdis, count);

    // cleanup test data
    client_guarantor
        .delete_guarantee_unchecked(&client_account)
        .await?;
    client_guarantor
        .delete_idf_all_unchecked(&word.kind)
        .await?;

    // ensure that the guarantee client has been unregistered
    assert!(client.get_idf_count_unchecked(&word).await.is_err());

    Ok(())
}
