use ipis::path::Path;

#[test]
fn test_create() {
    // create a static path to be stored
    let path = Path {
        value: "FjL3dTmyrudvLxFcezJ7b3oGq7Q48ZUS8HH5e4wajVL7"
            .parse()
            .unwrap(),
        len: 496_300_196,
    };

    // create a name to refer to a path
    let name = "my model";
}
