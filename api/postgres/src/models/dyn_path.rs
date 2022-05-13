#[derive(Queryable)]
pub struct DynPaths {
    pub id: i32,
    pub account: String,
    pub signature: String,
    pub name: String,
    pub path: String,
    pub len: i64,
}
