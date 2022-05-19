#![feature(more_qualified_paths)]

use bytecheck::CheckBytes;
use ipiis_common::{define_io, external_call, Ipiis, ServerResult};
use ipis::{
    async_trait::async_trait,
    core::{
        account::{AccountRef, GuaranteeSigned, GuarantorSigned},
        anyhow::{bail, Result},
        signed::IsSigned,
        value::word::WordHash,
    },
    path::{DynPath, Path},
};
use rkyv::{Archive, Deserialize, Serialize};

#[async_trait]
pub trait Ipdis {
    async fn ensure_registered(&self, guarantee: &AccountRef, guarantor: &AccountRef)
        -> Result<()>;

    async fn add_guarantee(&self, target: &GuaranteeSigned<AccountRef>) -> Result<()> {
        let guarantee = &target.guarantee.account;
        let guarantor = &target.data.guarantor;
        self.ensure_registered(guarantee, guarantee).await?;
        self.ensure_registered(guarantee, guarantor).await?;

        self.add_guarantee_unchecked(target).await
    }

    async fn add_guarantee_unchecked(&self, guarantee: &GuaranteeSigned<AccountRef>) -> Result<()>;

    async fn get_dyn_path<Path>(
        &self,
        path: &GuaranteeSigned<DynPath<Path>>,
    ) -> Result<Option<GuarantorSigned<DynPath<::ipis::path::Path>>>>
    where
        Path: Copy + Send + Sync,
    {
        let guarantee = &path.guarantee.account;
        let guarantor = &path.data.guarantor;
        self.ensure_registered(guarantee, guarantor).await?;

        self.get_dyn_path_unchecked(Some(guarantee), &path.data)
            .await
    }

    async fn get_dyn_path_unchecked<Path>(
        &self,
        guarantee: Option<&AccountRef>,
        path: &DynPath<Path>,
    ) -> Result<Option<GuarantorSigned<DynPath<::ipis::path::Path>>>>
    where
        Path: Copy + Send + Sync;

    async fn put_dyn_path(&self, path: &GuaranteeSigned<DynPath<Path>>) -> Result<()> {
        let guarantee = &path.guarantee.account;
        let guarantor = &path.data.guarantor;
        self.ensure_registered(guarantee, guarantor).await?;

        self.put_dyn_path_unchecked(path).await
    }

    async fn put_dyn_path_unchecked(&self, path: &GuaranteeSigned<DynPath<Path>>) -> Result<()>;

    async fn get_idf_count(&self, word: &GuaranteeSigned<WordHash>) -> Result<usize> {
        let guarantee = &word.guarantee.account;
        let guarantor = &word.data.guarantor;
        self.ensure_registered(guarantee, guarantor).await?;

        self.get_idf_count_unchecked(&word.data).await
    }

    async fn get_idf_count_unchecked(&self, word: &WordHash) -> Result<usize>;

    async fn get_idf_count_with_guarantee(
        &self,
        word: &GuaranteeSigned<WordHash>,
    ) -> Result<usize> {
        let guarantee = &word.guarantee.account;
        let guarantor = &word.data.guarantor;
        self.ensure_registered(guarantee, guarantor).await?;

        self.get_idf_count_with_guarantee_unchecked(guarantee, &word.data)
            .await
    }

    async fn get_idf_count_with_guarantee_unchecked(
        &self,
        guarantee: &AccountRef,
        word: &WordHash,
    ) -> Result<usize>;

    async fn get_idf_log(
        &self,
        word: &GuaranteeSigned<WordHash>,
    ) -> Result<Option<GuarantorSigned<WordHash>>> {
        let guarantee = &word.guarantee.account;
        let guarantor = &word.data.guarantor;
        self.ensure_registered(guarantee, guarantor).await?;

        self.get_idf_log_unchecked(Some(guarantee), &word.data)
            .await
    }

    async fn get_idf_log_unchecked(
        &self,
        guarantee: Option<&AccountRef>,
        word: &WordHash,
    ) -> Result<Option<GuarantorSigned<WordHash>>> {
        let query = GetIdfWords {
            word: *word,
            start_index: 0,
            end_index: 1,
        };

        self.get_idf_logs_unchecked(guarantee, &query)
            .await
            .map(|mut records| records.pop())
    }

    async fn get_idf_logs(
        &self,
        query: &GuaranteeSigned<GetIdfWords>,
    ) -> Result<Vec<GuarantorSigned<WordHash>>> {
        let guarantee = &query.guarantee.account;
        let guarantor = &query.data.guarantor;
        self.ensure_registered(guarantee, guarantor).await?;

        self.get_idf_logs_unchecked(Some(guarantee), &query.data)
            .await
    }

    async fn get_idf_logs_unchecked(
        &self,
        guarantee: Option<&AccountRef>,
        query: &GetIdfWords,
    ) -> Result<Vec<GuarantorSigned<WordHash>>>;

    async fn put_idf_log(&self, word: &GuaranteeSigned<WordHash>) -> Result<()> {
        let guarantee = &word.guarantee.account;
        let guarantor = &word.data.guarantor;
        self.ensure_registered(guarantee, guarantor).await?;

        self.put_idf_log_unchecked(word).await
    }

    async fn put_idf_log_unchecked(&self, word: &GuaranteeSigned<WordHash>) -> Result<()>;
}

#[async_trait]
impl<IpiisClient> Ipdis for IpiisClient
where
    IpiisClient: Ipiis + Send + Sync,
{
    async fn ensure_registered(
        &self,
        guarantee: &AccountRef,
        _guarantor: &AccountRef,
    ) -> Result<()> {
        let guarantee_now = self.account_me().account_ref();
        if guarantee != &guarantee_now {
            bail!("failed to authenticate the guarantee")
        }

        Ok(())
    }

    async fn add_guarantee_unchecked(&self, guarantee: &GuaranteeSigned<AccountRef>) -> Result<()> {
        // next target
        let target = self.get_account_primary(KIND.as_ref()).await?;

        // external call
        let () = external_call!(
            client: self,
            target: KIND.as_ref() => &target,
            request: crate::io => GuaranteePut,
            sign: *guarantee,
            inputs: { },
            outputs: { },
        );

        // unpack response
        Ok(())
    }

    async fn get_dyn_path_unchecked<Path>(
        &self,
        _guarantee: Option<&AccountRef>,
        path: &DynPath<Path>,
    ) -> Result<Option<GuarantorSigned<DynPath<::ipis::path::Path>>>>
    where
        Path: Copy + Send + Sync,
    {
        // next target
        let target = self.get_account_primary(KIND.as_ref()).await?;

        // external call
        let (path,) = external_call!(
            client: self,
            target: KIND.as_ref() => &target,
            request: crate::io => DynPathGet,
            sign: self.sign(target, (*path).remove_path())?,
            inputs: { },
            outputs: { path, },
        );

        // unpack response
        Ok(path)
    }

    async fn put_dyn_path_unchecked(&self, path: &GuaranteeSigned<DynPath<Path>>) -> Result<()> {
        // next target
        let target = self.get_account_primary(KIND.as_ref()).await?;

        // external call
        let () = external_call!(
            client: self,
            target: KIND.as_ref() => &target,
            request: crate::io => DynPathPut,
            sign: *path,
            inputs: { },
            outputs: { },
        );

        // unpack response
        Ok(())
    }

    async fn get_idf_count_unchecked(&self, word: &WordHash) -> Result<usize> {
        // next target
        let target = self.get_account_primary(KIND.as_ref()).await?;

        // external call
        let (count,) = external_call!(
            client: self,
            target: KIND.as_ref() => &target,
            request: crate::io => IdfCountGet,
            sign: self.sign(target, *word)?,
            inputs: { },
            outputs: { count, },
        );

        // unpack response
        count.try_into().map_err(Into::into)
    }

    async fn get_idf_count_with_guarantee_unchecked(
        &self,
        _guarantee: &AccountRef,
        word: &WordHash,
    ) -> Result<usize> {
        // next target
        let target = self.get_account_primary(KIND.as_ref()).await?;

        // external call
        let (count,) = external_call!(
            client: self,
            target: KIND.as_ref() => &target,
            request: crate::io => IdfCountGetWithGuarantee,
            sign: self.sign(target, *word)?,
            inputs: { },
            outputs: { count, },
        );

        // unpack response
        count.try_into().map_err(Into::into)
    }

    async fn get_idf_logs_unchecked(
        &self,
        _guarantee: Option<&AccountRef>,
        query: &GetIdfWords,
    ) -> Result<Vec<GuarantorSigned<WordHash>>> {
        // next target
        let target = self.get_account_primary(KIND.as_ref()).await?;

        // external call
        let (logs,) = external_call!(
            client: self,
            target: KIND.as_ref() => &target,
            request: crate::io => IdfLogGetMany,
            sign: self.sign(target, *query)?,
            inputs: { },
            outputs: { logs, },
        );

        // unpack response
        Ok(logs)
    }

    async fn put_idf_log_unchecked(&self, word: &GuaranteeSigned<WordHash>) -> Result<()> {
        // next target
        let target = self.get_account_primary(KIND.as_ref()).await?;

        // external call
        let () = external_call!(
            client: self,
            target: KIND.as_ref() => &target,
            request: crate::io => IdfLogPut,
            sign: *word,
            inputs: { },
            outputs: { },
        );

        // unpack response
        Ok(())
    }
}

define_io! {
    GuaranteePut {
        inputs: { },
        input_sign: GuaranteeSigned<AccountRef>,
        outputs: { },
        output_sign: GuarantorSigned<AccountRef>,
        generics: { },
    },
    DynPathGet {
        inputs: { },
        input_sign: GuaranteeSigned<DynPath<()>>,
        outputs: {
            path: Option<GuarantorSigned<DynPath<Path>>>,
        },
        output_sign: GuarantorSigned<DynPath<()>>,
        generics: { },
    },
    DynPathPut {
        inputs: { },
        input_sign: GuaranteeSigned<DynPath<Path>>,
        outputs: { },
        output_sign: GuarantorSigned<DynPath<Path>>,
        generics: { },
    },
    IdfCountGet {
        inputs: { },
        input_sign: GuaranteeSigned<WordHash>,
        outputs: {
            count: u32,
        },
        output_sign: GuarantorSigned<WordHash>,
        generics: { },
    },
    IdfCountGetWithGuarantee {
        inputs: { },
        input_sign: GuaranteeSigned<WordHash>,
        outputs: {
            count: u32,
        },
        output_sign: GuarantorSigned<WordHash>,
        generics: { },
    },
    IdfLogGetMany {
        inputs: { },
        input_sign: GuaranteeSigned<GetIdfWords>,
        outputs: {
            logs: Vec<GuarantorSigned<WordHash>>,
        },
        output_sign: GuarantorSigned<GetIdfWords>,
        generics: { },
    },
    IdfLogPut {
        inputs: { },
        input_sign: GuaranteeSigned<WordHash>,
        outputs: { },
        output_sign: GuarantorSigned<WordHash>,
        generics: { },
    },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug, PartialEq))]
pub struct GetIdfWords {
    pub word: WordHash,
    pub start_index: u32,
    pub end_index: u32,
}

impl IsSigned for GetIdfWords {}

::ipis::lazy_static::lazy_static! {
    pub static ref KIND: Option<::ipis::core::value::hash::Hash> = Some(
        ::ipis::core::value::hash::Hash::with_str("__ipis__ipdis__"),
    );
}
