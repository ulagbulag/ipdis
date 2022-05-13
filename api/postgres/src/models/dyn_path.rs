#[derive(Queryable)]
pub struct DynPaths {
    pub id: i32,
    pub name: String,
    pub path: String,
    pub len: i64,
}
