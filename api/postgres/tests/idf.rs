use ipdis_api_postgres::client::IpdisClient;
use ipdis_common::{ipiis_api::client::IpiisClient, Ipdis};
use ipiis_common::Ipiis;
use ipis::{
    core::value::{text::Text, word::Word},
    env::Infer,
    tokio,
};

#[tokio::test]
async fn test_create() {
    // create a client
    let client = IpdisClient::infer();
    let ipiis: &IpiisClient = client.as_ref();
    let account = ipiis.account_me().account_ref();

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
        let word = ipiis.sign(account, word).unwrap();

        // put the word in IPDIS
        client.put_idf_log_unsafe(&word).await.unwrap();
    }

    // get the word log
    let word_from_ipdis = client
        .get_idf_log_unsafe(None, &word)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(&word_from_ipdis.data.data.data, &word,);

    // get the word counts
    let count_from_ipdis = client.get_idf_count_unsafe(&word).await.unwrap();
    assert_eq!(count_from_ipdis, count);

    // cleanup test data
    client.delete_idf_all_unsafe(&word.kind).await.unwrap()
}
