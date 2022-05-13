use diesel::{Connection, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};
use ipiis_common::Ipiis;
use ipis::{
    core::{
        anyhow::{bail, Result},
        value::hash::Hash,
    },
    env::{self, Infer},
    path::{DynPath, Path},
};
use ipsis_api::client::IpsisClientInner;

pub type IpdisClient = IpdisClientInner<::ipdis_common::ipiis_api::client::IpiisClient>;

pub struct IpdisClientInner<IpiisClient> {
    pub ipsis: IpsisClientInner<IpiisClient>,
    connection: PgConnection,
}

impl<IpiisClient> AsRef<::ipdis_common::ipiis_api::client::IpiisClient>
    for IpdisClientInner<IpiisClient>
where
    IpiisClient: AsRef<::ipdis_common::ipiis_api::client::IpiisClient>,
{
    fn as_ref(&self) -> &::ipdis_common::ipiis_api::client::IpiisClient {
        self.ipsis.as_ref()
    }
}

impl<IpiisClient> AsRef<::ipdis_common::ipiis_api::server::IpiisServer>
    for IpdisClientInner<IpiisClient>
where
    IpiisClient: AsRef<::ipdis_common::ipiis_api::server::IpiisServer>,
{
    fn as_ref(&self) -> &::ipdis_common::ipiis_api::server::IpiisServer {
        self.ipsis.as_ref()
    }
}

impl<IpiisClient> AsRef<IpsisClientInner<IpiisClient>> for IpdisClientInner<IpiisClient> {
    fn as_ref(&self) -> &IpsisClientInner<IpiisClient> {
        &self.ipsis
    }
}

impl<'a> Infer<'a> for IpdisClient {
    type GenesisArgs = ();

    type GenesisResult = Self;

    fn try_infer() -> Result<Self>
    where
        Self: Sized,
    {
        let database_url: String = env::infer("DATABASE_URL")?;

        Ok(Self {
            ipsis: IpsisClientInner::try_infer()?,
            connection: PgConnection::establish(&database_url)
                .or_else(|_| bail!("Error connecting to {}", database_url))?,
        })
    }

    fn genesis((): <Self as Infer<'a>>::GenesisArgs) -> Result<<Self as Infer<'a>>::GenesisResult> {
        Self::try_infer()
    }
}

impl<IpiisClient> IpdisClientInner<IpiisClient>
where
    IpiisClient: AsRef<::ipdis_common::ipiis_api::client::IpiisClient>,
{
    pub async fn get_dyn_unsafe<Path>(
        &self,
        path: &DynPath<Path>,
    ) -> Result<Vec<crate::models::dyn_paths::DynPath>> {
        let account = self.ipsis.as_ref().account_me().account_ref();

        crate::schema::dyn_paths::table
            .filter(crate::schema::dyn_paths::account.eq(account.to_string()))
            .filter(crate::schema::dyn_paths::kind.eq(path.kind.to_string()))
            .filter(crate::schema::dyn_paths::word.eq(path.word.to_string()))
            .get_results(&self.connection)
            .map_err(Into::into)
    }

    pub async fn put_dyn(&self, path: &DynPath<Path>) -> Result<crate::models::dyn_paths::DynPath> {
        let path = self
            .ipsis
            .as_ref()
            .sign(self.ipsis.as_ref().account_me().account_ref(), *path)?;

        let record = crate::models::dyn_paths::NewDynPath {
            account: path.guarantee.account.to_string(),
            signature: path.guarantee.signature.to_string(),
            created_date: path.created_date.naive_utc(),
            expiration_date: path.expiration_date.map(|e| e.naive_utc()),
            kind: path.data.kind.to_string(),
            word: path.data.word.to_string(),
            path: path.data.path.value.to_string(),
            len: path.data.path.len.try_into()?,
        };

        ::diesel::insert_into(crate::schema::dyn_paths::table)
            .values(&record)
            .get_result(&self.connection)
            .map_err(Into::into)
    }

    pub async fn delete_dyn_all_unsafe(&self, kind: &Hash) -> Result<usize> {
        ::diesel::delete(crate::schema::dyn_paths::table)
            .filter(crate::schema::dyn_paths::kind.eq(kind.to_string()))
            .execute(&self.connection)
            .map_err(Into::into)
    }
}
