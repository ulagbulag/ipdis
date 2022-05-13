use diesel::{Connection, PgConnection};
use ipis::{
    core::anyhow::{bail, Result},
    env::{self, Infer},
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
