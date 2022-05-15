use diesel::{
    dsl::now, BoolExpressionMethods, Connection, ExpressionMethods, PgConnection, QueryDsl,
    RunQueryDsl,
};
use ipdis_common::{GetIdfWords, Ipdis};
use ipiis_common::Ipiis;
use ipis::{
    async_trait::async_trait,
    core::{
        account::{AccountRef, GuaranteeSigned, GuarantorSigned, Identity},
        anyhow::{bail, Result},
        metadata::Metadata,
        value::{chrono::NaiveDateTime, hash::Hash, text::TextHash, uuid::Uuid, word::WordHash},
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

impl<'a, IpiisClient> Infer<'a> for IpdisClientInner<IpiisClient>
where
    IpiisClient: Infer<'a, GenesisResult = IpiisClient>,
    <IpiisClient as Infer<'a>>::GenesisArgs: Sized,
{
    type GenesisArgs = <IpiisClient as Infer<'a>>::GenesisArgs;
    type GenesisResult = Self;

    fn try_infer() -> Result<Self>
    where
        Self: Sized,
    {
        IpsisClientInner::try_infer().and_then(Self::with_ipsis_client)
    }

    fn genesis(
        args: <Self as Infer<'a>>::GenesisArgs,
    ) -> Result<<Self as Infer<'a>>::GenesisResult> {
        IpsisClientInner::genesis(args).and_then(Self::with_ipsis_client)
    }
}

impl<IpiisClient> IpdisClientInner<IpiisClient> {
    pub fn with_ipsis_client(ipsis: IpsisClientInner<IpiisClient>) -> Result<Self> {
        let database_url: String = env::infer("DATABASE_URL")?;

        Ok(Self {
            ipsis,
            connection: PgConnection::establish(&database_url)
                .or_else(|_| bail!("Error connecting to {}", database_url))?
                .into(),
        })
    }
}

#[async_trait]
impl<IpiisClient> Ipdis for IpdisClientInner<IpiisClient>
where
    IpiisClient: AsRef<::ipdis_common::ipiis_api::client::IpiisClient> + Send + Sync,
{
    async fn ensure_registered(
        &self,
        guarantee: &AccountRef,
        guarantor: &AccountRef,
    ) -> Result<()> {
        let guarantor_now = self.ipsis.as_ref().account_me().account_ref();
        if guarantor != &guarantor_now {
            bail!("failed to authenticate the guarantor")
        }

        // skip authentication for self-authentication
        if guarantee == guarantor {
            return Ok(());
        }

        crate::schema::accounts_guarantees::table
            .filter(crate::schema::accounts_guarantees::guarantee.eq(guarantee.to_string()))
            .filter(crate::schema::accounts_guarantees::guarantor.eq(guarantor.to_string()))
            .filter(crate::schema::accounts_guarantees::created_date.lt(now))
            .filter(
                crate::schema::accounts_guarantees::expiration_date
                    .ge(now)
                    .or(crate::schema::accounts_guarantees::expiration_date.is_null()),
            )
            .execute(&mut *self.connection.lock().await)
            .map_err(Into::into)
            .and_then(|count| {
                if count > 0 {
                    Ok(())
                } else {
                    bail!("failed to authenticate the guarantee")
                }
            })
    }

    async fn add_guarantee_unchecked(&self, guarantee: &GuaranteeSigned<AccountRef>) -> Result<()> {
        let guarantee = self.ipsis.as_ref().sign_as_guarantor(*guarantee)?;

        let record = crate::models::accounts_guarantees::NewAccountsGuarantee {
            nonce: guarantee.nonce.0 .0,
            guarantee: guarantee.guarantee.account.to_string(),
            guarantor: guarantee.guarantor.account.to_string(),
            guarantee_signature: guarantee.guarantee.signature.to_string(),
            guarantor_signature: guarantee.guarantor.signature.to_string(),
            created_date: guarantee.created_date.naive_utc(),
            expiration_date: guarantee.expiration_date.map(|e| e.naive_utc()),
        };

        ::diesel::insert_into(crate::schema::accounts_guarantees::table)
            .values(&record)
            .execute(&mut *self.connection.lock().await)
            .map(|_| ())
            .map_err(Into::into)
    }

    async fn get_dyn_path_unchecked<Path>(
        &self,
        guarantee: Option<&AccountRef>,
        path: &DynPath<Path>,
    ) -> Result<Option<GuarantorSigned<DynPath<::ipis::path::Path>>>>
    where
        Path: Copy + Send + Sync,
    {
        let guarantor = self.ipsis.as_ref().account_me().account_ref();
        let guarantee = guarantee.unwrap_or(&guarantor);

        let mut records: Vec<crate::models::dyn_paths::DynPath> = crate::schema::dyn_paths::table
            .filter(crate::schema::dyn_paths::guarantee.eq(guarantee.to_string()))
            .filter(crate::schema::dyn_paths::guarantor.eq(guarantor.to_string()))
            .filter(crate::schema::dyn_paths::created_date.lt(now))
            .filter(
                crate::schema::dyn_paths::expiration_date
                    .ge(now)
                    .or(crate::schema::dyn_paths::expiration_date.is_null()),
            )
            .filter(crate::schema::dyn_paths::kind.eq(path.kind.to_string()))
            .filter(crate::schema::dyn_paths::word.eq(path.word.to_string()))
            .get_results(&mut *self.connection.lock().await)?;

        match records.pop() {
            Some(record) => Ok(Some(GuarantorSigned {
                guarantor: Identity {
                    account: AccountRef {
                        public_key: record.guarantor.parse()?,
                    },
                    signature: record.guarantor_signature.parse()?,
                },
                data: GuaranteeSigned {
                    guarantee: Identity {
                        account: AccountRef {
                            public_key: record.guarantee.parse()?,
                        },
                        signature: record.guarantee_signature.parse()?,
                    },
                    data: Metadata {
                        nonce: Uuid(record.nonce).into(),
                        created_date: NaiveDateTime(record.created_date).to_utc(),
                        expiration_date: record.expiration_date.map(|e| NaiveDateTime(e).to_utc()),
                        guarantor: record.guarantor.parse()?,
                        data: DynPath {
                            kind: record.kind.parse()?,
                            word: record.word.parse()?,
                            path: ::ipis::path::Path {
                                value: record.path.parse()?,
                                len: record.len.try_into()?,
                            },
                        },
                    },
                },
            })),
            None => Ok(None),
        }
    }

    async fn put_dyn_path_unchecked(&self, path: &GuaranteeSigned<DynPath<Path>>) -> Result<()> {
        let path = self.ipsis.as_ref().sign_as_guarantor(*path)?;

        let record = crate::models::dyn_paths::NewDynPath {
            nonce: path.nonce.0 .0,
            guarantee: path.guarantee.account.to_string(),
            guarantor: path.guarantor.account.to_string(),
            guarantee_signature: path.guarantee.signature.to_string(),
            guarantor_signature: path.guarantor.signature.to_string(),
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

    async fn get_idf_count_unchecked(&self, word: &WordHash) -> Result<usize> {
        match crate::schema::idf_words::table
            .filter(crate::schema::idf_words::kind.eq(word.kind.to_string()))
            .filter(crate::schema::idf_words::lang.eq(word.text.lang.to_string()))
            .filter(crate::schema::idf_words::word.eq(word.text.msg.to_string()))
            .get_results::<crate::models::idf::IdfWord>(&mut *self.connection.lock().await)?
            .pop()
        {
            Some(record) => record.count.try_into().map_err(Into::into),
            None => Ok(0),
        }
    }

    async fn get_idf_count_with_guarantee_unchecked(
        &self,
        guarantee: &AccountRef,
        word: &WordHash,
    ) -> Result<usize> {
        match crate::schema::idf_words_guarantees::table
            .filter(crate::schema::idf_words_guarantees::guarantee.eq(guarantee.to_string()))
            .filter(crate::schema::idf_words_guarantees::kind.eq(word.kind.to_string()))
            .filter(crate::schema::idf_words_guarantees::lang.eq(word.text.lang.to_string()))
            .filter(crate::schema::idf_words_guarantees::word.eq(word.text.msg.to_string()))
            .get_results::<crate::models::idf::IdfWordGuarantee>(
                &mut *self.connection.lock().await,
            )?
            .pop()
        {
            Some(record) => record.count.try_into().map_err(Into::into),
            None => Ok(0),
        }
    }

    async fn get_idf_logs_unchecked(
        &self,
        guarantee: Option<&AccountRef>,
        query: &GetIdfWords,
    ) -> Result<Vec<GuarantorSigned<WordHash>>> {
        let guarantor = self.ipsis.as_ref().account_me().account_ref();
        let guarantee = guarantee.unwrap_or(&guarantor);

        let records: Vec<crate::models::idf::IdfLog> = crate::schema::idf_logs::table
            .filter(crate::schema::idf_logs::guarantee.eq(guarantee.to_string()))
            .filter(crate::schema::idf_logs::guarantor.eq(guarantor.to_string()))
            .filter(crate::schema::idf_logs::created_date.lt(now))
            .filter(
                crate::schema::idf_logs::expiration_date
                    .ge(now)
                    .or(crate::schema::idf_logs::expiration_date.is_null()),
            )
            .filter(crate::schema::idf_logs::kind.eq(query.word.kind.to_string()))
            .filter(crate::schema::idf_logs::lang.eq(query.word.text.lang.to_string()))
            .filter(crate::schema::idf_logs::word.eq(query.word.text.msg.to_string()))
            .get_results(&mut *self.connection.lock().await)?;

        records
            .into_iter()
            .map(|record| {
                Ok(GuarantorSigned {
                    guarantor: Identity {
                        account: AccountRef {
                            public_key: record.guarantor.parse()?,
                        },
                        signature: record.guarantor_signature.parse()?,
                    },
                    data: GuaranteeSigned {
                        guarantee: Identity {
                            account: AccountRef {
                                public_key: record.guarantee.parse()?,
                            },
                            signature: record.guarantee_signature.parse()?,
                        },
                        data: Metadata {
                            nonce: Uuid(record.nonce).into(),
                            created_date: NaiveDateTime(record.created_date).to_utc(),
                            expiration_date: record
                                .expiration_date
                                .map(|e| NaiveDateTime(e).to_utc()),
                            guarantor: record.guarantor.parse()?,
                            data: WordHash {
                                kind: record.kind.parse()?,
                                text: TextHash {
                                    lang: record.lang.parse()?,
                                    msg: record.word.parse()?,
                                },
                            },
                        },
                    },
                })
            })
            .collect()
    }

    async fn put_idf_log_unchecked(&self, word: &GuaranteeSigned<WordHash>) -> Result<()> {
        let word = self.ipsis.as_ref().sign_as_guarantor(*word)?;

        let record = crate::models::idf::NewIdfLog {
            nonce: word.nonce.0 .0,
            guarantee: word.guarantee.account.to_string(),
            guarantor: word.guarantor.account.to_string(),
            guarantee_signature: word.guarantee.signature.to_string(),
            guarantor_signature: word.guarantor.signature.to_string(),
            created_date: word.created_date.naive_utc(),
            expiration_date: word.expiration_date.map(|e| e.naive_utc()),
            kind: word.data.kind.to_string(),
            lang: word.data.text.lang.to_string(),
            word: word.data.text.msg.to_string(),
        };

        self.connection
            .lock()
            .await
            .transaction::<(), ::diesel::result::Error, _>(|conn| {
                // insert the log record
                ::diesel::insert_into(crate::schema::idf_logs::table)
                    .values(&record)
                    .execute(conn)?;

                // check whether word exists
                match crate::schema::idf_words::table
                    .filter(crate::schema::idf_words::kind.eq(&record.kind))
                    .filter(crate::schema::idf_words::lang.eq(&record.lang))
                    .filter(crate::schema::idf_words::word.eq(&record.word))
                    .get_results::<crate::models::idf::IdfWord>(conn)?
                    .pop()
                {
                    // old word => append the count
                    Some(idf_word) => ::diesel::update(crate::schema::idf_words::table)
                        .filter(crate::schema::idf_words::id.eq(idf_word.id))
                        .set(crate::schema::idf_words::count.eq(idf_word.count + 1))
                        .execute(conn)?,
                    // new word => insert the word record
                    None => {
                        let word_record = crate::models::idf::NewIdfWord {
                            kind: record.kind.clone(),
                            lang: record.lang.clone(),
                            word: record.word.clone(),
                            count: 1,
                        };

                        ::diesel::insert_into(crate::schema::idf_words::table)
                            .values(&word_record)
                            .execute(conn)?
                    }
                };

                // check whether word of guarantee exists
                match crate::schema::idf_words_guarantees::table
                    .filter(crate::schema::idf_words_guarantees::guarantee.eq(&record.guarantee))
                    .filter(crate::schema::idf_words_guarantees::kind.eq(&record.kind))
                    .filter(crate::schema::idf_words_guarantees::lang.eq(&record.lang))
                    .filter(crate::schema::idf_words_guarantees::word.eq(&record.word))
                    .get_results::<crate::models::idf::IdfWordGuarantee>(conn)?
                    .pop()
                {
                    // old word => append the count
                    Some(idf_word) => ::diesel::update(crate::schema::idf_words_guarantees::table)
                        .filter(crate::schema::idf_words_guarantees::id.eq(idf_word.id))
                        .set(crate::schema::idf_words_guarantees::count.eq(idf_word.count + 1))
                        .execute(conn)?,
                    // new word => insert the word record
                    None => {
                        let word_record = crate::models::idf::NewIdfWordGuarantee {
                            guarantee: record.guarantee.clone(),
                            kind: record.kind.clone(),
                            lang: record.lang.clone(),
                            word: record.word.clone(),
                            count: 1,
                        };

                        ::diesel::insert_into(crate::schema::idf_words_guarantees::table)
                            .values(&word_record)
                            .execute(conn)?
                    }
                };

                Ok(())
            })
            .map_err(Into::into)
    }
}

impl<IpiisClient> IpdisClientInner<IpiisClient>
where
    IpiisClient: AsRef<::ipdis_common::ipiis_api::client::IpiisClient>,
{
    pub async fn delete_guarantee_unchecked(&self, guarantee: &AccountRef) -> Result<()> {
        ::diesel::delete(crate::schema::accounts_guarantees::table)
            .filter(crate::schema::accounts_guarantees::guarantee.eq(guarantee.to_string()))
            .execute(&mut *self.connection.lock().await)
            .map(|_| ())
            .map_err(Into::into)
    }

    pub async fn delete_dyn_path_all_unchecked(&self, kind: &Hash) -> Result<()> {
        ::diesel::delete(crate::schema::dyn_paths::table)
            .filter(crate::schema::dyn_paths::kind.eq(kind.to_string()))
            .execute(&mut *self.connection.lock().await)
            .map(|_| ())
            .map_err(Into::into)
    }

    pub async fn delete_idf_all_unchecked(&self, kind: &Hash) -> Result<()> {
        self.connection
            .lock()
            .await
            .transaction::<(), ::diesel::result::Error, _>(|conn| {
                ::diesel::delete(crate::schema::idf_words::table)
                    .filter(crate::schema::idf_words::kind.eq(kind.to_string()))
                    .execute(conn)
                    .map(|_| ())?;

                ::diesel::delete(crate::schema::idf_words_guarantees::table)
                    .filter(crate::schema::idf_words_guarantees::kind.eq(kind.to_string()))
                    .execute(conn)
                    .map(|_| ())?;

                ::diesel::delete(crate::schema::idf_logs::table)
                    .filter(crate::schema::idf_logs::kind.eq(kind.to_string()))
                    .execute(conn)
                    .map(|_| ())?;

                Ok(())
            })
            .map_err(Into::into)
    }
}
