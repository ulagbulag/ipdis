pub extern crate ipiis_api;

use bytecheck::CheckBytes;
use ipis::{
    async_trait::async_trait,
    core::{
        account::{AccountRef, GuaranteeSigned, GuarantorSigned},
        anyhow::Result,
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

        self.add_guarantee_unsafe(target).await
    }

    async fn add_guarantee_unsafe(&self, guarantee: &GuaranteeSigned<AccountRef>) -> Result<()>;

    async fn get_dyn_path<Path>(
        &self,
        path: &GuaranteeSigned<DynPath<Path>>,
    ) -> Result<Option<GuarantorSigned<DynPath<::ipis::path::Path>>>>
    where
        Path: Send + Sync,
    {
        let guarantee = &path.guarantee.account;
        let guarantor = &path.data.guarantor;
        self.ensure_registered(guarantee, guarantor).await?;

        self.get_dyn_path_unsafe(Some(guarantee), &path.data).await
    }

    async fn get_dyn_path_unsafe<Path>(
        &self,
        guarantee: Option<&AccountRef>,
        path: &DynPath<Path>,
    ) -> Result<Option<GuarantorSigned<DynPath<::ipis::path::Path>>>>
    where
        Path: Send + Sync;

    async fn put_dyn_path(&self, path: &GuaranteeSigned<DynPath<Path>>) -> Result<()> {
        let guarantee = &path.guarantee.account;
        let guarantor = &path.data.guarantor;
        self.ensure_registered(guarantee, guarantor).await?;

        self.put_dyn_path_unsafe(path).await
    }

    async fn put_dyn_path_unsafe(&self, path: &GuaranteeSigned<DynPath<Path>>) -> Result<()>;

    async fn get_idf_count(&self, word: &GuaranteeSigned<WordHash>) -> Result<usize> {
        let guarantee = &word.guarantee.account;
        let guarantor = &word.data.guarantor;
        self.ensure_registered(guarantee, guarantor).await?;

        self.get_idf_count_unsafe(&word.data).await
    }

    async fn get_idf_count_unsafe(&self, word: &WordHash) -> Result<usize>;

    async fn get_idf_log(
        &self,
        word: &GuaranteeSigned<WordHash>,
    ) -> Result<Option<GuarantorSigned<WordHash>>> {
        let guarantee = &word.guarantee.account;
        let guarantor = &word.data.guarantor;
        self.ensure_registered(guarantee, guarantor).await?;

        self.get_idf_log_unsafe(Some(guarantee), &word.data).await
    }

    async fn get_idf_log_unsafe(
        &self,
        guarantee: Option<&AccountRef>,
        word: &WordHash,
    ) -> Result<Option<GuarantorSigned<WordHash>>> {
        let query = GetIdfWords {
            word: *word,
            start_index: 0,
            end_index: 1,
        };

        self.get_idf_logs_unsafe(guarantee, &query)
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

        self.get_idf_logs_unsafe(Some(guarantee), &query.data).await
    }

    async fn get_idf_logs_unsafe(
        &self,
        guarantee: Option<&AccountRef>,
        query: &GetIdfWords,
    ) -> Result<Vec<GuarantorSigned<WordHash>>>;

    async fn put_idf_log(&self, word: &GuaranteeSigned<WordHash>) -> Result<()> {
        let guarantee = &word.guarantee.account;
        let guarantor = &word.data.guarantor;
        self.ensure_registered(guarantee, guarantor).await?;

        self.put_idf_log_unsafe(word).await
    }

    async fn put_idf_log_unsafe(&self, word: &GuaranteeSigned<WordHash>) -> Result<()>;
}

#[derive(Clone, Debug, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug, PartialEq))]
pub struct GetIdfWords {
    pub word: WordHash,
    pub start_index: u32,
    pub end_index: u32,
}
