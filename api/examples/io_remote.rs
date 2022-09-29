use ipdis_api::{
    client::IpdisClient,
    common::{Ipdis, KIND},
    server::IpdisServer,
};
use ipdis_common::{GetWords, GetWordsCounts, GetWordsParent};
use ipiis_api::{client::IpiisClient, common::Ipiis, server::IpiisServer};
use ipis::{
    core::{
        anyhow::Result,
        value::{hash::Hash, text::Text},
    },
    env::{infer, Infer},
    path::Path,
    tokio,
    word::{Word, WordHash, WordKey},
};

#[tokio::main]
async fn main() -> Result<()> {
    // deploy a server
    let server = IpdisServer::genesis(9801).await?;
    let server_account = {
        let server: &IpiisServer = server.as_ref();
        let account = unsafe { server.account_me()? };

        // register the environment variables
        ::std::env::set_var("ipis_account_me", account.to_string());

        account.account_ref()
    };
    tokio::spawn(async move { server.run().await });

    // create a guarantor client
    let client_guarantor = IpdisClient::infer().await;

    // create a client
    let client = IpiisClient::genesis(None).await?;
    let client_account = client.account_ref();
    client
        .set_account_primary(KIND.as_ref(), &server_account)
        .await?;
    client
        .set_address(KIND.as_ref(), &server_account, &infer("ipiis_client_account_primary_address")?)
        .await?;

    // cleanup client registration
    client_guarantor
        .delete_guarantee_unchecked(client_account)
        .await?;

    // register the client as guarantee
    {
        // sign as guarantor
        let guarantee = client.sign_owned(server_account, *client_account)?;

        client_guarantor.add_guarantee_unchecked(&guarantee).await?;
    };

    // create a sample word to be stored
    let namespace = "ipdis-api-postgres-test";
    let kind = "ipdis-api-postgres-test";
    let parent = "";
    let word = Word {
        key: WordKey {
            namespace: namespace.to_string(),
            text: Text::with_en_us("hello world"),
        },
        kind: kind.to_string(),
        relpath: true,
        path: Path {
            value: "bafkreihm64a7oj6z4llxysvetldpxpgjs4tyvsqbbpo65olbyegpktkdli"
                .parse()
                .unwrap(),
            len: 13,
        },
    };

    // make it hash
    let word: WordHash = word.into();
    let parent = Hash::with_str(parent);
    let parent_word = {
        let mut word = word;
        word.key.text.msg = parent;
        word
    };

    // cleanup test data
    client_guarantor
        .delete_word_all_unchecked(&word.key.namespace)
        .await
        .unwrap();

    // put the word in IPDIS (* 3 times)
    let count = 3u32;
    for _ in 0..count {
        // sign as guarantee
        let word = client.sign_owned(server_account, word).unwrap();

        // put the word in IPDIS
        client.put_word_unchecked(&parent, &word).await.unwrap();
    }

    // get the words
    let word_from_ipdis = client
        .get_word_latest_unchecked(None, &word.key)
        .await?
        .unwrap();
    assert_eq!(&word_from_ipdis.data, &word);

    // get the parent's words
    let words_from_ipdis = client
        .get_word_many_unchecked(
            None,
            &GetWords {
                word: parent_word.key,
                parent: GetWordsParent::Duplicated,
                start_index: 0,
                end_index: 1,
            },
        )
        .await
        .unwrap();
    assert_eq!(&words_from_ipdis[0].data, &word);

    // get the word counts
    let count_from_ipdis = client
        .get_word_count_unchecked(None, &word.key, false)
        .await?;
    assert_eq!(count_from_ipdis, count);

    // get the word counts of the account
    let count_from_ipdis = client
        .get_word_count_unchecked(None, &word.key, true)
        .await?;
    assert_eq!(count_from_ipdis, count);

    // get the parent's word counts
    assert_eq!(
        client
            .get_word_count_many_unchecked(
                None,
                &GetWordsCounts {
                    word: parent_word.key,
                    parent: true,
                    owned: false,
                    start_index: 0,
                    end_index: 1,
                }
            )
            .await
            .unwrap()
            .pop()
            .unwrap()
            .count,
        count,
    );

    // get the parent's word counts of the account
    assert_eq!(
        client
            .get_word_count_many_unchecked(
                None,
                &GetWordsCounts {
                    word: parent_word.key,
                    parent: true,
                    owned: true,
                    start_index: 0,
                    end_index: 1,
                }
            )
            .await
            .unwrap()
            .pop()
            .unwrap()
            .count,
        count,
    );

    // cleanup test data
    client_guarantor
        .delete_guarantee_unchecked(client_account)
        .await?;
    client_guarantor
        .delete_word_all_unchecked(&word.key.namespace)
        .await?;

    // ensure that the guarantee client has been unregistered
    assert_eq!(
        client_guarantor
            .get_word_count_unchecked(None, &word.key, false)
            .await?,
        0,
    );

    Ok(())
}
