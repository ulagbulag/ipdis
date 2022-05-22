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
        WordGetMany => handle_word_get_many,
        WordCountGetMany => handle_word_count_get_many,
        WordPut => handle_word_put,
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

    async fn handle_word_get_many(
        client: &IpdisClientInner<IpiisServer>,
        req: ::ipdis_common::io::request::WordGetMany<'static>,
    ) -> Result<::ipdis_common::io::response::WordGetMany<'static>> {
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
        let words = client
            .get_word_many_unchecked(Some(guarantee), &query)
            .await?;

        // sign data
        let server: &IpiisServer = client.as_ref();
        let sign = server.sign_as_guarantor(sign_as_guarantee)?;

        // pack data
        Ok(::ipdis_common::io::response::WordGetMany {
            __lifetime: Default::default(),
            __sign: ::ipis::stream::DynStream::Owned(sign),
            words: ::ipis::stream::DynStream::Owned(words),
        })
    }

    async fn handle_word_count_get_many(
        client: &IpdisClientInner<IpiisServer>,
        req: ::ipdis_common::io::request::WordCountGetMany<'static>,
    ) -> Result<::ipdis_common::io::response::WordCountGetMany<'static>> {
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
        let counts = client
            .get_word_count_many_unchecked(Some(guarantee), &query)
            .await?;

        // sign data
        let server: &IpiisServer = client.as_ref();
        let sign = server.sign_as_guarantor(sign_as_guarantee)?;

        // pack data
        Ok(::ipdis_common::io::response::WordCountGetMany {
            __lifetime: Default::default(),
            __sign: ::ipis::stream::DynStream::Owned(sign),
            counts: ::ipis::stream::DynStream::Owned(counts),
        })
    }

    async fn handle_word_put(
        client: &IpdisClientInner<IpiisServer>,
        req: ::ipdis_common::io::request::WordPut<'static>,
    ) -> Result<::ipdis_common::io::response::WordPut<'static>> {
        // unpack sign
        let sign_as_guarantee = req.__sign.into_owned().await?;

        // ensure registered
        let guarantee = &sign_as_guarantee.guarantee.account;
        client
            .ensure_registered(guarantee, &sign_as_guarantee.guarantor)
            .await?;

        // unpack data
        let parent = req.parent.into_owned().await?;

        // handle data
        let () = client
            .put_word_unchecked(&parent, &sign_as_guarantee)
            .await?;

        // sign data
        let server: &IpiisServer = client.as_ref();
        let sign = server.sign_as_guarantor(sign_as_guarantee)?;

        // pack data
        Ok(::ipdis_common::io::response::WordPut {
            __lifetime: Default::default(),
            __sign: ::ipis::stream::DynStream::Owned(sign),
        })
    }
}
