use diesel::{Connection, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};
use ipiis_common::Ipiis;
use ipis::{
    core::{
        account::{AccountRef, GuaranteeSigned, Identity},
        anyhow::{bail, Result},
        metadata::Metadata,
        value::{chrono::NaiveDateTime, hash::Hash, uuid::Uuid},
    },
    env::{self, Infer},
    path::{DynPath, Path},
    tokio::sync::Mutex,
};
use ipsis_api::client::IpsisClientInner;

pub type IpdisClient = IpdisClientInner<::ipdis_common::ipiis_api::client::IpiisClient>;

pub struct IpdisClientInner<IpiisClient> {
    pub ipsis: IpsisClientInner<IpiisClient>,
    connection: Mutex<PgConnection>,
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
                .or_else(|_| bail!("Error connecting to {}", database_url))?
                .into(),
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
    ) -> Result<Vec<GuaranteeSigned<DynPath<::ipis::path::Path>>>> {
        let account = self.ipsis.as_ref().account_me().account_ref();

        crate::schema::dyn_paths::table
            .filter(crate::schema::dyn_paths::guarantee.eq(account.to_string()))
            .filter(crate::schema::dyn_paths::kind.eq(path.kind.to_string()))
            .filter(crate::schema::dyn_paths::word.eq(path.word.to_string()))
            .get_results(&mut *self.connection.lock().await)?
            .into_iter()
            .map(|row: crate::models::dyn_paths::DynPath| {
                Ok(GuaranteeSigned {
                    guarantee: Identity {
                        account: AccountRef {
                            public_key: row.guarantee.parse()?,
                        },
                        signature: row.signature.parse()?,
                    },
                    data: Metadata {
                        nonce: Uuid(row.nonce).into(),
                        created_date: NaiveDateTime(row.created_date).to_utc(),
                        expiration_date: row.expiration_date.map(|e| NaiveDateTime(e).to_utc()),
                        guarantor: row.guarantor.parse()?,
                        data: DynPath {
                            kind: row.kind.parse()?,
                            word: row.word.parse()?,
                            path: ::ipis::path::Path {
                                value: row.path.parse()?,
                                len: row.len.try_into()?,
                            },
                        },
                    },
                })
            })
            .collect()
    }

    pub async fn put_dyn(&self, path: &DynPath<Path>) -> Result<()> {
        let path = self
            .ipsis
            .as_ref()
            .sign(self.ipsis.as_ref().account_me().account_ref(), *path)?;

        let record = crate::models::dyn_paths::NewDynPath {
            nonce: path.nonce.0 .0,
            guarantee: path.guarantee.account.to_string(),
            guarantor: path.guarantor.to_string(),
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
            .execute(&mut *self.connection.lock().await)
            .map(|_| ())
            .map_err(Into::into)
    }

    pub async fn delete_dyn_all_unsafe(&self, kind: &Hash) -> Result<()> {
        ::diesel::delete(crate::schema::dyn_paths::table)
            .filter(crate::schema::dyn_paths::kind.eq(kind.to_string()))
            .execute(&mut *self.connection.lock().await)
            .map(|_| ())
            .map_err(Into::into)
    }
}
