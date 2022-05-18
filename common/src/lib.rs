#![feature(more_qualified_paths)]

use bytecheck::CheckBytes;
use ipiis_common::{external_call, Ipiis};
use ipis::{
    async_trait::async_trait,
    core::{
        account::{AccountRef, GuaranteeSigned, GuarantorSigned},
        anyhow::{bail, Result},
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

        // pack request
        let req = RequestType::GuaranteePut {
            guarantee: (*guarantee).into(),
        };

        // external call
        let () = external_call!(
            call: self
                .call_permanent_deserialized(&target, req)
                .await?,
            response: Response => GuaranteePut,
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

        // pack request
        let req = RequestType::DynPathGet {
            path: (*path).remove_path().into(),
        };

        // external call
        let (path,) = external_call!(
            call: self
                .call_permanent_deserialized(&target, req)
                .await?,
            response: Response => DynPathGet,
            items: { path },
        );

        // unpack response
        Ok(*path)
    }

    async fn put_dyn_path_unchecked(&self, path: &GuaranteeSigned<DynPath<Path>>) -> Result<()> {
        // next target
        let target = self.get_account_primary(KIND.as_ref()).await?;

        // pack request
        let req = RequestType::DynPathPut {
            path: (*path).into(),
        };

        // external call
        let () = external_call!(
            call: self
                .call_permanent_deserialized(&target, req)
                .await?,
            response: Response => DynPathPut,
        );

        // unpack response
        Ok(())
    }

    async fn get_idf_count_unchecked(&self, word: &WordHash) -> Result<usize> {
        // next target
        let target = self.get_account_primary(KIND.as_ref()).await?;

        // pack request
        let req = RequestType::IdfCountGet {
            word: (*word).into(),
        };

        // external call
        let (count,) = external_call!(
            call: self
                .call_permanent_deserialized(&target, req)
                .await?,
            response: Response => IdfCountGet,
            items: { count },
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

        // pack request
        let req = RequestType::IdfCountGetWithGuarantee {
            word: (*word).into(),
        };

        // external call
        let (count,) = external_call!(
            call: self
                .call_permanent_deserialized(&target, req)
                .await?,
            response: Response => IdfCountGetWithGuarantee,
            items: { count },
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

        // pack request
        let req = RequestType::IdfLogGetMany {
            query: (*query).into(),
        };

        // external call
        let (logs,) = external_call!(
            call: self
                .call_permanent_deserialized(&target, req)
                .await?,
            response: Response => IdfLogGetMany,
            items: { logs },
        );

        // unpack response
        Ok(logs)
    }

    async fn put_idf_log_unchecked(&self, word: &GuaranteeSigned<WordHash>) -> Result<()> {
        // next target
        let target = self.get_account_primary(KIND.as_ref()).await?;

        // pack request
        let req = RequestType::IdfLogPut {
            word: (*word).into(),
        };

        // external call
        let () = external_call!(
            call: self
                .call_permanent_deserialized(&target, req)
                .await?,
            response: Response => IdfLogPut,
        );

        // unpack response
        Ok(())
    }
}

pub type Request = GuaranteeSigned<RequestType>;

#[derive(Clone, Debug, PartialEq, Archive, Serialize, Deserialize)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug, PartialEq))]
pub enum RequestType {
    GuaranteePut {
        guarantee: Box<GuaranteeSigned<AccountRef>>,
    },
    DynPathGet {
        path: Box<DynPath<()>>,
    },
    DynPathPut {
        path: Box<GuaranteeSigned<DynPath<Path>>>,
    },
    IdfCountGet {
        word: Box<WordHash>,
    },
    IdfCountGetWithGuarantee {
        word: Box<WordHash>,
    },
    IdfLogGetMany {
        query: Box<GetIdfWords>,
    },
    IdfLogPut {
        word: Box<GuaranteeSigned<WordHash>>,
    },
}

#[derive(Clone, Debug, PartialEq, Archive, Serialize, Deserialize)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug, PartialEq))]
pub enum Response {
    GuaranteePut,
    DynPathGet {
        path: Box<Option<GuarantorSigned<DynPath<Path>>>>,
    },
    DynPathPut,
    IdfCountGet {
        count: u32,
    },
    IdfCountGetWithGuarantee {
        count: u32,
    },
    IdfLogGetMany {
        logs: Vec<GuarantorSigned<WordHash>>,
    },
    IdfLogPut,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug, PartialEq))]
pub struct GetIdfWords {
    pub word: WordHash,
    pub start_index: u32,
    pub end_index: u32,
}

::ipis::lazy_static::lazy_static! {
    pub static ref KIND: Option<::ipis::core::value::hash::Hash> = Some(
        ::ipis::core::value::hash::Hash::with_str("__ipis__ipdis__"),
    );
}
