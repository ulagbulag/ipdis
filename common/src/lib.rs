use bytecheck::CheckBytes;
use ipiis_common::{define_io, external_call, Ipiis, ServerResult};
use ipis::{
    async_trait::async_trait,
    core::{
        account::{AccountRef, GuaranteeSigned, GuarantorSigned},
        anyhow::{bail, Result},
        signed::IsSigned,
        value::hash::Hash,
    },
    path::{DynPath, Path},
    word::{WordHash, WordKeyHash},
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

    async fn get_word_latest(
        &self,
        word: &GuaranteeSigned<WordKeyHash>,
    ) -> Result<Option<GuarantorSigned<WordHash>>> {
        let guarantee = &word.guarantee.account;
        let guarantor = &word.data.guarantor;
        self.ensure_registered(guarantee, guarantor).await?;

        self.get_word_latest_unchecked(Some(guarantee), &word.data)
            .await
    }

    async fn get_word_latest_unchecked(
        &self,
        guarantee: Option<&AccountRef>,
        word: &WordKeyHash,
    ) -> Result<Option<GuarantorSigned<WordHash>>> {
        let query = GetWords {
            word: *word,
            parent: GetWordsParent::None,
            start_index: 0,
            end_index: 1,
        };

        self.get_word_many_unchecked(guarantee, &query)
            .await
            .map(|mut records| records.pop())
    }

    async fn get_word_many(
        &self,
        query: &GuaranteeSigned<GetWords>,
    ) -> Result<Vec<GuarantorSigned<WordHash>>> {
        let guarantee = &query.guarantee.account;
        let guarantor = &query.data.guarantor;
        self.ensure_registered(guarantee, guarantor).await?;

        self.get_word_many_unchecked(Some(guarantee), &query.data)
            .await
    }

    async fn get_word_many_unchecked(
        &self,
        guarantee: Option<&AccountRef>,
        query: &GetWords,
    ) -> Result<Vec<GuarantorSigned<WordHash>>>;

    async fn get_word_count(
        &self,
        word: &GuaranteeSigned<WordKeyHash>,
        owned: bool,
    ) -> Result<u32> {
        let guarantee = &word.guarantee.account;
        let guarantor = &word.data.guarantor;
        self.ensure_registered(guarantee, guarantor).await?;

        self.get_word_count_unchecked(Some(guarantee), &word.data, owned)
            .await
    }

    async fn get_word_count_unchecked(
        &self,
        guarantee: Option<&AccountRef>,
        word: &WordKeyHash,
        owned: bool,
    ) -> Result<u32> {
        let query = GetWordsCounts {
            word: *word,
            parent: false,
            owned,
            start_index: 0,
            end_index: 1,
        };

        self.get_word_count_many_unchecked(guarantee, &query)
            .await
            .map(|mut records| records.pop().map(|record| record.count).unwrap_or(0))
    }

    async fn get_word_count_many(
        &self,
        query: &GuaranteeSigned<GetWordsCounts>,
    ) -> Result<Vec<GetWordsCountsOutput>> {
        let guarantee = &query.guarantee.account;
        let guarantor = &query.data.guarantor;
        self.ensure_registered(guarantee, guarantor).await?;

        self.get_word_count_many_unchecked(Some(guarantee), &query.data)
            .await
    }

    async fn get_word_count_many_unchecked(
        &self,
        guarantee: Option<&AccountRef>,
        query: &GetWordsCounts,
    ) -> Result<Vec<GetWordsCountsOutput>>;

    async fn put_word(&self, parent: &Hash, word: &GuaranteeSigned<WordHash>) -> Result<()> {
        let guarantee = &word.guarantee.account;
        let guarantor = &word.data.guarantor;
        self.ensure_registered(guarantee, guarantor).await?;

        self.put_word_unchecked(parent, word).await
    }

    async fn put_word_unchecked(
        &self,
        parent: &Hash,
        word: &GuaranteeSigned<WordHash>,
    ) -> Result<()>;
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
        let guarantee_now = self.account_ref();
        if guarantee != guarantee_now {
            bail!("failed to authenticate the guarantee")
        }

        Ok(())
    }

    async fn add_guarantee_unchecked(&self, guarantee: &GuaranteeSigned<AccountRef>) -> Result<()> {
        // next target
        let target = self.get_account_primary(KIND.as_ref()).await?;

        // external call
        external_call!(
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
        external_call!(
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

    async fn get_word_many_unchecked(
        &self,
        _guarantee: Option<&AccountRef>,
        query: &GetWords,
    ) -> Result<Vec<GuarantorSigned<WordHash>>> {
        // next target
        let target = self.get_account_primary(KIND.as_ref()).await?;

        // external call
        let (words,) = external_call!(
            client: self,
            target: KIND.as_ref() => &target,
            request: crate::io => WordGetMany,
            sign: self.sign(target, *query)?,
            inputs: { },
            outputs: { words, },
        );

        // unpack response
        Ok(words)
    }

    async fn get_word_count_many_unchecked(
        &self,
        _guarantee: Option<&AccountRef>,
        query: &GetWordsCounts,
    ) -> Result<Vec<GetWordsCountsOutput>> {
        // next target
        let target = self.get_account_primary(KIND.as_ref()).await?;

        // external call
        let (counts,) = external_call!(
            client: self,
            target: KIND.as_ref() => &target,
            request: crate::io => WordCountGetMany,
            sign: self.sign(target, *query)?,
            inputs: { },
            outputs: { counts, },
        );

        // unpack response
        Ok(counts)
    }

    async fn put_word_unchecked(
        &self,
        parent: &Hash,
        word: &GuaranteeSigned<WordHash>,
    ) -> Result<()> {
        // next target
        let target = self.get_account_primary(KIND.as_ref()).await?;

        // external call
        external_call!(
            client: self,
            target: KIND.as_ref() => &target,
            request: crate::io => WordPut,
            sign: *word,
            inputs: {
                parent: *parent,
            },
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
    WordGetMany {
        inputs: { },
        input_sign: GuaranteeSigned<GetWords>,
        outputs: {
            words: Vec<GuarantorSigned<WordHash>>,
        },
        output_sign: GuarantorSigned<GetWords>,
        generics: { },
    },
    WordPut {
        inputs: {
            parent: Hash,
        },
        input_sign: GuaranteeSigned<WordHash>,
        outputs: { },
        output_sign: GuarantorSigned<WordHash>,
        generics: { },
    },
    WordCountGetMany {
        inputs: { },
        input_sign: GuaranteeSigned<GetWordsCounts>,
        outputs: {
            counts: Vec<GetWordsCountsOutput>,
        },
        output_sign: GuarantorSigned<GetWordsCounts>,
        generics: { },
    },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug, PartialEq))]
pub struct GetWords {
    pub word: WordKeyHash,
    pub parent: GetWordsParent,
    /// inclusive left bound
    pub start_index: u32,
    /// exclusive right bound
    pub end_index: u32,
}

impl IsSigned for GetWords {}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug, PartialEq, Eq))]
pub enum GetWordsParent {
    None,
    Duplicated,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug, PartialEq))]
pub struct GetWordsCounts {
    pub word: WordKeyHash,
    pub parent: bool,
    pub owned: bool,
    /// inclusive left bound
    pub start_index: u32,
    /// exclusive right bound
    pub end_index: u32,
}

impl IsSigned for GetWordsCounts {}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug, PartialEq))]
pub struct GetWordsCountsOutput {
    pub word: GetWordKeyHash,
    pub count: u32,
}

impl IsSigned for GetWordsCountsOutput {}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug, PartialEq))]
pub struct GetWordKeyHash {
    pub key: WordKeyHash,
    pub kind: Hash,
}

impl IsSigned for GetWordKeyHash {}

::ipis::lazy_static::lazy_static! {
    pub static ref KIND: Option<::ipis::core::value::hash::Hash> = Some(
        ::ipis::core::value::hash::Hash::with_str("__ipis__ipdis__"),
    );
}
