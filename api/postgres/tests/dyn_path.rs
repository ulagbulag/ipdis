use ipdis_api_postgres::client::IpdisClient;
use ipis::{
    core::value::hash::Hash,
    env::Infer,
    path::{DynPath, Path},
    tokio,
};

#[tokio::test]
async fn test_create() {
    // create a client
    let client = IpdisClient::infer();

    // create a static path to be stored
    let path = Path {
        value: "FjL3dTmyrudvLxFcezJ7b3oGq7Q48ZUS8HH5e4wajVL7"
            .parse()
            .unwrap(),
        len: 496_300_196,
    };

    // create a pair of kind & word to refer to a path
    let kind = "ipdis-api-postgres-test";
    let word = "my model";

    // create a dynamic path
    let dyn_path = DynPath {
        kind: Hash::with_str(kind),
        word: Hash::with_str(word),
        path,
    };

    // put the path in IPDIS
    client.put_dyn(&dyn_path).await.unwrap();

    // cleanup test data
    assert!(client.delete_dyn_all(&dyn_path.kind).await.unwrap() > 0);
}
