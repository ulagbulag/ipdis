use ipdis_api::client::IpdisClient;
use ipdis_common::Ipdis;
use ipiis_api::{client::IpiisClient, common::Ipiis};
use ipis::{
    core::value::hash::Hash,
    env::Infer,
    path::{DynPath, Path},
    tokio,
};

#[tokio::test]
async fn test_create() {
    // create a client
    let client = IpdisClient::infer().await;
    let ipiis: &IpiisClient = client.as_ref();
    let account = ipiis.account_ref();

    // create a static path to be stored
    let path = Path {
        value: "bafybeie52ly6uafpr4h3ih24mqa4twtojppo6366kyi74ejtd4sxv2fezm"
            .parse()
            .unwrap(),
        len: 496_300_196,
    };

    // create a pair of kind & word to refer to a path
    let namespace = "ipdis-api-postgres-test";
    let kind = "ipdis-api-postgres-test";
    let word = "my model";

    // create a dynamic path
    let dyn_path = DynPath {
        namespace: Hash::with_str(namespace),
        kind: Hash::with_str(kind),
        word: Hash::with_str(word),
        path,
    };

    // cleanup test data
    client
        .delete_dyn_path_all_unchecked(&dyn_path.kind)
        .await
        .unwrap();

    // sign as guarantee
    let dyn_path = ipiis.sign_owned(*account, dyn_path).unwrap();

    // put the path in IPDIS
    client.put_dyn_path_unchecked(&dyn_path).await.unwrap();

    // get the path
    let dyn_path_from_ipdis = client
        .get_dyn_path_unchecked(None, &dyn_path.remove_path())
        .await
        .unwrap()
        .unwrap();
    // FIXME: precision issue (postgres != rust chrono)
    // assert_eq!(&dyn_path_from_ipdis.metadata.data, &dyn_path.metadata,);
    assert_eq!(&dyn_path_from_ipdis.data, &dyn_path.data,);

    // cleanup test data
    client
        .delete_dyn_path_all_unchecked(&dyn_path.namespace)
        .await
        .unwrap()
}
