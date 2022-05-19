use std::sync::Arc;

use ipdis_common::Ipdis;
use ipiis_api::{
    client::IpiisClient,
    common::{handle_external_call, Ipiis, ServerResult},
    server::IpiisServer,
};
use ipis::{async_trait::async_trait, core::anyhow::Result, env::Infer};

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

handle_external_call!(
    server: IpdisServer => IpdisClientInner<IpiisServer>,
    name: run,
    request: ::ipdis_common::io => {
        GuaranteePut => handle_guarantee_put,
        DynPathGet => handle_dyn_path_get,
        DynPathPut => handle_dyn_path_put,
        IdfCountGet => handle_idf_count_get,
        IdfCountGetWithGuarantee => handle_idf_count_get_with_guarantee,
        IdfLogGetMany => handle_log_get_many,
        IdfLogPut => handle_log_put,
    },
);

impl IpdisServer {
    async fn handle_guarantee_put(
        client: &IpdisClientInner<IpiisServer>,
        req: ::ipdis_common::io::request::GuaranteePut<'static>,
    ) -> Result<::ipdis_common::io::response::GuaranteePut<'static>> {
        // unpack sign
        let sign_as_guarantee = req.__sign.into_owned().await?;

        // ensure registered
        let guarantee = &sign_as_guarantee.guarantee.account;
        client
            .ensure_registered(guarantee, &sign_as_guarantee.guarantor)
            .await?;

        // handle data
        let () = client.add_guarantee_unchecked(&sign_as_guarantee).await?;

        // sign data
        let server: &IpiisServer = client.as_ref();
        let sign = server.sign_as_guarantor(sign_as_guarantee)?;

        // pack data
        Ok(::ipdis_common::io::response::GuaranteePut {
            __lifetime: Default::default(),
            __sign: ::ipis::stream::DynStream::Owned(sign),
        })
    }

    async fn handle_dyn_path_get(
        client: &IpdisClientInner<IpiisServer>,
        req: ::ipdis_common::io::request::DynPathGet<'static>,
    ) -> Result<::ipdis_common::io::response::DynPathGet<'static>> {
        // unpack sign
        let sign_as_guarantee = req.__sign.into_owned().await?;

        // ensure registered
        let guarantee = &sign_as_guarantee.guarantee.account;
        client
            .ensure_registered(guarantee, &sign_as_guarantee.guarantor)
            .await?;

        // unpack data
        let path = sign_as_guarantee.data.data;

        // handle data
        let path = client
            .get_dyn_path_unchecked(Some(guarantee), &path)
            .await?;

        // sign data
        let server: &IpiisServer = client.as_ref();
        let sign = server.sign_as_guarantor(sign_as_guarantee)?;

        // pack data
        Ok(::ipdis_common::io::response::DynPathGet {
            __lifetime: Default::default(),
            __sign: ::ipis::stream::DynStream::Owned(sign),
            path: ::ipis::stream::DynStream::Owned(path),
        })
    }

    async fn handle_dyn_path_put(
        client: &IpdisClientInner<IpiisServer>,
        req: ::ipdis_common::io::request::DynPathPut<'static>,
    ) -> Result<::ipdis_common::io::response::DynPathPut<'static>> {
        // unpack sign
        let sign_as_guarantee = req.__sign.into_owned().await?;

        // ensure registered
        let guarantee = &sign_as_guarantee.guarantee.account;
        client
            .ensure_registered(guarantee, &sign_as_guarantee.guarantor)
            .await?;

        // handle data
        let () = client.put_dyn_path_unchecked(&sign_as_guarantee).await?;

        // sign data
        let server: &IpiisServer = client.as_ref();
        let sign = server.sign_as_guarantor(sign_as_guarantee)?;

        // pack data
        Ok(::ipdis_common::io::response::DynPathPut {
            __lifetime: Default::default(),
            __sign: ::ipis::stream::DynStream::Owned(sign),
        })
    }

    async fn handle_idf_count_get(
        client: &IpdisClientInner<IpiisServer>,
        req: ::ipdis_common::io::request::IdfCountGet<'static>,
    ) -> Result<::ipdis_common::io::response::IdfCountGet<'static>> {
        // unpack sign
        let sign_as_guarantee = req.__sign.into_owned().await?;

        // ensure registered
        let guarantee = &sign_as_guarantee.guarantee.account;
        client
            .ensure_registered(guarantee, &sign_as_guarantee.guarantor)
            .await?;

        // unpack data
        let word = sign_as_guarantee.data.data;

        // handle data
        let count = client.get_idf_count_unchecked(&word).await?;

        // sign data
        let server: &IpiisServer = client.as_ref();
        let sign = server.sign_as_guarantor(sign_as_guarantee)?;

        // pack data
        Ok(::ipdis_common::io::response::IdfCountGet {
            __lifetime: Default::default(),
            __sign: ::ipis::stream::DynStream::Owned(sign),
            count: ::ipis::stream::DynStream::Owned(count.try_into()?),
        })
    }

    async fn handle_idf_count_get_with_guarantee(
        client: &IpdisClientInner<IpiisServer>,
        req: ::ipdis_common::io::request::IdfCountGetWithGuarantee<'static>,
    ) -> Result<::ipdis_common::io::response::IdfCountGetWithGuarantee<'static>> {
        // unpack sign
        let sign_as_guarantee = req.__sign.into_owned().await?;

        // ensure registered
        let guarantee = &sign_as_guarantee.guarantee.account;
        client
            .ensure_registered(guarantee, &sign_as_guarantee.guarantor)
            .await?;

        // handle data
        let count = client
            .get_idf_count_with_guarantee(&sign_as_guarantee)
            .await?;

        // sign data
        let server: &IpiisServer = client.as_ref();
        let sign = server.sign_as_guarantor(sign_as_guarantee)?;

        // pack data
        Ok(::ipdis_common::io::response::IdfCountGetWithGuarantee {
            __lifetime: Default::default(),
            __sign: ::ipis::stream::DynStream::Owned(sign),
            count: ::ipis::stream::DynStream::Owned(count.try_into()?),
        })
    }

    async fn handle_log_get_many(
        client: &IpdisClientInner<IpiisServer>,
        req: ::ipdis_common::io::request::IdfLogGetMany<'static>,
    ) -> Result<::ipdis_common::io::response::IdfLogGetMany<'static>> {
        // unpack sign
        let sign_as_guarantee = req.__sign.into_owned().await?;

        // ensure registered
        let guarantee = &sign_as_guarantee.guarantee.account;
        client
            .ensure_registered(guarantee, &sign_as_guarantee.guarantor)
            .await?;

        // unpack data
        let query = sign_as_guarantee.data.data;

        // handle data
        let logs = client
            .get_idf_logs_unchecked(Some(guarantee), &query)
            .await?;

        // sign data
        let server: &IpiisServer = client.as_ref();
        let sign = server.sign_as_guarantor(sign_as_guarantee)?;

        // pack data
        Ok(::ipdis_common::io::response::IdfLogGetMany {
            __lifetime: Default::default(),
            __sign: ::ipis::stream::DynStream::Owned(sign),
            logs: ::ipis::stream::DynStream::Owned(logs),
        })
    }

    async fn handle_log_put(
        client: &IpdisClientInner<IpiisServer>,
        req: ::ipdis_common::io::request::IdfLogPut<'static>,
    ) -> Result<::ipdis_common::io::response::IdfLogPut<'static>> {
        // unpack sign
        let sign_as_guarantee = req.__sign.into_owned().await?;

        // ensure registered
        let guarantee = &sign_as_guarantee.guarantee.account;
        client
            .ensure_registered(guarantee, &sign_as_guarantee.guarantor)
            .await?;

        // handle data
        let () = client.put_idf_log_unchecked(&sign_as_guarantee).await?;

        // sign data
        let server: &IpiisServer = client.as_ref();
        let sign = server.sign_as_guarantor(sign_as_guarantee)?;

        // pack data
        Ok(::ipdis_common::io::response::IdfLogPut {
            __lifetime: Default::default(),
            __sign: ::ipis::stream::DynStream::Owned(sign),
        })
    }
}
