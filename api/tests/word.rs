use ipdis_api::{
    client::IpdisClient,
    common::{GetWords, GetWordsCounts, GetWordsParent, Ipdis},
};
use ipiis_api::{client::IpiisClient, common::Ipiis};
use ipis::{
    core::value::{hash::Hash, text::Text},
    env::Infer,
    path::Path,
    tokio,
    word::{Word, WordHash, WordKey},
};

#[tokio::test]
async fn test_create() {
    // create a client
    let client = IpdisClient::infer().await;
    let ipiis: &IpiisClient = client.as_ref();
    let account = ipiis.account_me().account_ref();

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
            value: "Gx1fwyQphUwVU5E2HRbx7H6t7QVZ8XsMzrFz6TnyxaC1"
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
    client
        .delete_word_all_unchecked(&word.key.namespace)
        .await
        .unwrap();

    // put the word in IPDIS (* 3 times)
    let count = 3u32;
    for _ in 0..count {
        // sign as guarantee
        let word = ipiis.sign(account, word).unwrap();

        // put the word in IPDIS
        client.put_word_unchecked(&parent, &word).await.unwrap();
    }

    // get the words
    let word_from_ipdis = client
        .get_word_latest_unchecked(None, &word.key)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(&word_from_ipdis.data.data.data, &word);

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
    assert_eq!(&words_from_ipdis[0].data.data.data, &word);

    // get the word counts
    let count_from_ipdis = client
        .get_word_count_unchecked(None, &word.key, false)
        .await
        .unwrap();
    assert_eq!(count_from_ipdis, count);

    // get the word counts of the account
    let count_from_ipdis = client
        .get_word_count_unchecked(None, &word.key, true)
        .await
        .unwrap();
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
    client
        .delete_word_all_unchecked(&word.key.namespace)
        .await
        .unwrap();

    // ensure that the guarantee client has been unregistered
    assert_eq!(
        client
            .get_word_count_unchecked(None, &word.key, false)
            .await
            .unwrap(),
        0,
    );
}
