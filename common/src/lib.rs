pub extern crate ipiis_api;

use core::ops::RangeBounds;

use bytecheck::CheckBytes;
use ipis::{
    async_trait::async_trait,
    class::Class,
    core::{anyhow::Result, signature::SignatureSerializer},
    path::DynPath,
    pin::PinnedInner,
};
use rkyv::{
    de::deserializers::SharedDeserializeMap, validation::validators::DefaultValidator, Archive,
    Deserialize, Serialize,
};

#[async_trait]
pub trait Ipdis {
    async fn get_dyn<Res>(&self, path: &DynPath) -> Result<Res>
    where
        Res: Class
            + Archive
            + Serialize<SignatureSerializer>
            + ::core::fmt::Debug
            + PartialEq
            + Send,
        <Res as Archive>::Archived: for<'a> CheckBytes<DefaultValidator<'a>>
            + Deserialize<Res, SharedDeserializeMap>
            + ::core::fmt::Debug
            + PartialEq
            + Send,
    {
        {
            self.get_dyn_raw(path)
                .await
                .and_then(PinnedInner::deserialize_owned)
        }
    }

    async fn get_dyn_raw(&self, path: &DynPath) -> Result<Vec<u8>>;

    async fn get_ranked<Range, Res>(&self, path: &DynPath, range: Range) -> Result<Vec<Res>>
    where
        Range: RangeBounds<usize> + Send,
        Res: Class
            + Archive
            + Serialize<SignatureSerializer>
            + ::core::fmt::Debug
            + PartialEq
            + Send,
        <Res as Archive>::Archived: for<'a> CheckBytes<DefaultValidator<'a>>
            + Deserialize<Res, SharedDeserializeMap>
            + ::core::fmt::Debug
            + PartialEq
            + Send,
    {
        {
            self.get_ranked_raw(path, range).await.and_then(|rows| {
                rows.into_iter()
                    .map(PinnedInner::deserialize_owned)
                    .collect()
            })
        }
    }

    async fn get_ranked_raw<Range>(&self, path: &DynPath, range: Range) -> Result<Vec<Vec<u8>>>
    where
        Range: RangeBounds<usize>;
}
