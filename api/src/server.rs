use std::sync::Arc;

use ipdis_common::{Ipdis, Request, RequestType, Response};
use ipiis_api::server::IpiisServer;
use ipis::{async_trait::async_trait, core::anyhow::Result, env::Infer, pin::Pinned};

use crate::client::IpdisClientInner;

pub struct IpdisServer {
    client: Arc<IpdisClientInner<IpiisServer>>,
}

impl ::core::ops::Deref for IpdisServer {
    type Target = IpdisClientInner<IpiisServer>;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

#[async_trait]
impl<'a> Infer<'a> for IpdisServer {
    type GenesisArgs = <IpiisServer as Infer<'a>>::GenesisArgs;
    type GenesisResult = Self;

    async fn try_infer() -> Result<Self> {
        Ok(Self {
            client: IpdisClientInner::try_infer().await?.into(),
        })
    }

    async fn genesis(
        args: <Self as Infer<'a>>::GenesisArgs,
    ) -> Result<<Self as Infer<'a>>::GenesisResult> {
        Ok(Self {
            client: IpdisClientInner::genesis(args).await?.into(),
        })
    }
}

impl IpdisServer {
    pub async fn run(&self) {
        let client = self.client.clone();

        let runtime: &IpiisServer = (*self.client).as_ref();
        runtime.run(client, Self::handle).await
    }

    async fn handle(
        client: Arc<IpdisClientInner<IpiisServer>>,
        req: Pinned<Request>,
    ) -> Result<Response> {
        // TODO: CURD without deserializing
        let req = req.deserialize_into()?;
        let guarantee = &req.guarantee.account;
        client.ensure_registered(guarantee, &req.guarantor).await?;

        match req.data.data {
            RequestType::GuaranteePut { guarantee } => client
                .add_guarantee_unchecked(&guarantee)
                .await
                .map(|()| Response::GuaranteePut),
            RequestType::DynPathGet { path } => Ok(Response::DynPathGet {
                path: client
                    .get_dyn_path_unchecked(Some(guarantee), &path)
                    .await?
                    .into(),
            }),
            RequestType::DynPathPut { path } => client
                .put_dyn_path_unchecked(&path)
                .await
                .map(|()| Response::DynPathPut),
            RequestType::IdfCountGet { word } => Ok(Response::IdfCountGet {
                count: client.get_idf_count_unchecked(&word).await?.try_into()?,
            }),
            RequestType::IdfCountGetWithGuarantee { word } => {
                Ok(Response::IdfCountGetWithGuarantee {
                    count: client.get_idf_count_unchecked(&word).await?.try_into()?,
                })
            }
            RequestType::IdfLogGetMany { query } => Ok(Response::IdfLogGetMany {
                logs: client
                    .get_idf_logs_unchecked(Some(guarantee), &query)
                    .await?,
            }),
            RequestType::IdfLogPut { word } => client
                .put_idf_log_unchecked(&word)
                .await
                .map(|()| Response::IdfLogPut),
        }
    }
}
