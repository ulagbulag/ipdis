use diesel::{
    dsl::now, BoolExpressionMethods, Connection, ExpressionMethods, PgConnection, QueryDsl,
    RunQueryDsl,
};
use ipdis_common::{
    GetWordKeyHash, GetWords, GetWordsCounts, GetWordsCountsOutput, GetWordsParent, Ipdis,
};
use ipiis_api::common::Ipiis;
use ipis::{
    async_trait::async_trait,
    core::{
        account::{AccountRef, GuaranteeSigned, GuarantorSigned, Identity},
        anyhow::{bail, Result},
        metadata::Metadata,
        value::{chrono::NaiveDateTime, hash::Hash, text::TextHash, uuid::Uuid},
    },
    env::{self, Infer},
    path::{DynPath, Path},
    tokio::sync::Mutex,
    word::{WordHash, WordKeyHash},
};

pub type IpdisClient = IpdisClientInner<::ipiis_api::client::IpiisClient>;

pub struct IpdisClientInner<IpiisClient> {
    pub ipiis: IpiisClient,
    connection: Mutex<PgConnection>,
}

impl<IpiisClient> AsRef<::ipiis_api::client::IpiisClient> for IpdisClientInner<IpiisClient>
where
    IpiisClient: AsRef<::ipiis_api::client::IpiisClient>,
{
    fn as_ref(&self) -> &::ipiis_api::client::IpiisClient {
        self.ipiis.as_ref()
    }
}

impl<IpiisClient> AsRef<::ipiis_api::server::IpiisServer> for IpdisClientInner<IpiisClient>
where
    IpiisClient: AsRef<::ipiis_api::server::IpiisServer>,
{
    fn as_ref(&self) -> &::ipiis_api::server::IpiisServer {
        self.ipiis.as_ref()
    }
}

#[async_trait]
impl<'a, IpiisClient> Infer<'a> for IpdisClientInner<IpiisClient>
where
    Self: Send,
    IpiisClient: Infer<'a, GenesisResult = IpiisClient>,
    <IpiisClient as Infer<'a>>::GenesisArgs: Sized,
{
    type GenesisArgs = <IpiisClient as Infer<'a>>::GenesisArgs;
    type GenesisResult = Self;

    async fn try_infer() -> Result<Self>
    where
        Self: Sized,
    {
        IpiisClient::try_infer()
            .await
            .and_then(Self::with_ipiis_client)
    }

    async fn genesis(
        args: <Self as Infer<'a>>::GenesisArgs,
    ) -> Result<<Self as Infer<'a>>::GenesisResult> {
        IpiisClient::genesis(args)
            .await
            .and_then(Self::with_ipiis_client)
    }
}

impl<IpiisClient> IpdisClientInner<IpiisClient> {
    pub fn with_ipiis_client(ipiis: IpiisClient) -> Result<Self> {
        let database_url: String = env::infer("DATABASE_URL")?;

        Ok(Self {
            ipiis,
            connection: PgConnection::establish(&database_url)
                .or_else(|_| bail!("Error connecting to {database_url}"))?
                .into(),
        })
    }
}

#[async_trait]
impl<IpiisClient> Ipdis for IpdisClientInner<IpiisClient>
where
    IpiisClient: Ipiis + Send + Sync,
{
    async fn ensure_registered(
        &self,
        guarantee: &AccountRef,
        guarantor: &AccountRef,
    ) -> Result<()> {
        let guarantor_now = self.ipiis.account_me().account_ref();
        if guarantor != &guarantor_now {
            bail!("failed to authenticate the guarantor")
        }

        // skip authentication for self-authentication
        if guarantee == guarantor {
            return Ok(());
        }

        crate::schema::accounts_guarantees::table
            .limit(1)
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
        let guarantee = self.ipiis.sign_as_guarantor(*guarantee)?;

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
        let guarantor = self.ipiis.account_me().account_ref();
        let guarantee = guarantee.unwrap_or(&guarantor);

        let mut records: Vec<crate::models::dyn_paths::DynPath> = crate::schema::dyn_paths::table
            .order(crate::schema::dyn_paths::created_date.desc())
            .limit(1)
            .filter(crate::schema::dyn_paths::guarantee.eq(guarantee.to_string()))
            .filter(crate::schema::dyn_paths::guarantor.eq(guarantor.to_string()))
            .filter(crate::schema::dyn_paths::created_date.lt(now))
            .filter(
                crate::schema::dyn_paths::expiration_date
                    .ge(now)
                    .or(crate::schema::dyn_paths::expiration_date.is_null()),
            )
            .filter(crate::schema::dyn_paths::namespace.eq(path.namespace.to_string()))
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
                            namespace: record.namespace.parse()?,
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
        let path = self.ipiis.sign_as_guarantor(*path)?;

        let record = crate::models::dyn_paths::NewDynPath {
            nonce: path.nonce.0 .0,
            guarantee: path.guarantee.account.to_string(),
            guarantor: path.guarantor.account.to_string(),
            guarantee_signature: path.guarantee.signature.to_string(),
            guarantor_signature: path.guarantor.signature.to_string(),
            created_date: path.created_date.naive_utc(),
            expiration_date: path.expiration_date.map(|e| e.naive_utc()),
            namespace: path.data.namespace.to_string(),
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

    async fn get_word_many_unchecked(
        &self,
        guarantee: Option<&AccountRef>,
        query: &GetWords,
    ) -> Result<Vec<GuarantorSigned<WordHash>>> {
        if query.end_index <= query.start_index {
            bail!("malformed index: end_index should be bigger than start_index")
        }

        let guarantor = self.ipiis.account_me().account_ref();
        let guarantee = guarantee.unwrap_or(&guarantor);

        let sql = crate::schema::words::table
            .order(crate::schema::words::id.desc())
            // TODO: improve performance (pagination: rather than offset & limit ?)
            .offset(query.start_index.into())
            .limit((query.end_index - query.start_index).into())
            .filter(crate::schema::words::guarantee.eq(guarantee.to_string()))
            .filter(crate::schema::words::guarantor.eq(guarantor.to_string()))
            .filter(crate::schema::words::created_date.lt(now))
            .filter(
                crate::schema::words::expiration_date
                    .ge(now)
                    .or(crate::schema::words::expiration_date.is_null()),
            )
            .filter(crate::schema::words::namespace.eq(query.word.namespace.to_string()))
            .filter(crate::schema::words::lang.eq(query.word.text.lang.to_string()));

        let records: Vec<crate::models::words::Word> = match query.parent {
            GetWordsParent::None => sql
                .filter(crate::schema::words::word.eq(query.word.text.msg.to_string()))
                .get_results(&mut *self.connection.lock().await)?,
            GetWordsParent::Duplicated => sql
                .filter(crate::schema::words::parent.eq(query.word.text.msg.to_string()))
                .get_results(&mut *self.connection.lock().await)?,
        };

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
                                key: WordKeyHash {
                                    namespace: record.namespace.parse()?,
                                    text: TextHash {
                                        lang: record.lang.parse()?,
                                        msg: record.word.parse()?,
                                    },
                                },
                                kind: record.kind.parse()?,
                                relpath: record.relpath,
                                path: Path {
                                    value: record.path.parse()?,
                                    len: record.len.try_into()?,
                                },
                            },
                        },
                    },
                })
            })
            .collect()
    }

    async fn get_word_count_many_unchecked(
        &self,
        guarantee: Option<&AccountRef>,
        query: &GetWordsCounts,
    ) -> Result<Vec<GetWordsCountsOutput>> {
        let guarantor = self.ipiis.account_me().account_ref();
        let guarantee = guarantee.unwrap_or(&guarantor);

        if query.owned {
            let sql = crate::schema::words_counts_guarantees::table
                .order(crate::schema::words_counts_guarantees::id.desc())
                // TODO: improve performance (pagination: rather than offset & limit ?)
                .offset(query.start_index.into())
                .limit((query.end_index - query.start_index).into())
                .filter(crate::schema::words_counts_guarantees::guarantee.eq(guarantee.to_string()))
                .filter(
                    crate::schema::words_counts_guarantees::namespace
                        .eq(query.word.namespace.to_string()),
                )
                .filter(
                    crate::schema::words_counts_guarantees::lang.eq(query
                        .word
                        .text
                        .lang
                        .to_string()),
                );

            let records: Vec<crate::models::words::WordCountGuarantee> = if query.parent {
                sql.filter(
                    crate::schema::words_counts_guarantees::parent.eq(query
                        .word
                        .text
                        .msg
                        .to_string()),
                )
                .get_results(&mut *self.connection.lock().await)?
            } else {
                sql.filter(
                    crate::schema::words_counts_guarantees::word.eq(query
                        .word
                        .text
                        .msg
                        .to_string()),
                )
                .get_results(&mut *self.connection.lock().await)?
            };

            records
                .into_iter()
                .map(|record| {
                    Ok(GetWordsCountsOutput {
                        word: GetWordKeyHash {
                            key: WordKeyHash {
                                namespace: record.namespace.parse()?,
                                text: TextHash {
                                    lang: record.lang.parse()?,
                                    msg: record.word.parse()?,
                                },
                            },
                            kind: record.kind.parse()?,
                        },
                        count: record.count.try_into()?,
                    })
                })
                .collect()
        } else {
            let sql = crate::schema::words_counts::table
                .order(crate::schema::words_counts::id.desc())
                // TODO: improve performance (pagination: rather than offset & limit ?)
                .offset(query.start_index.into())
                .limit((query.end_index - query.start_index).into())
                .filter(crate::schema::words_counts::namespace.eq(query.word.namespace.to_string()))
                .filter(crate::schema::words_counts::lang.eq(query.word.text.lang.to_string()));

            let records: Vec<crate::models::words::WordCount> = if query.parent {
                sql.filter(crate::schema::words_counts::parent.eq(query.word.text.msg.to_string()))
                    .get_results(&mut *self.connection.lock().await)?
            } else {
                sql.filter(crate::schema::words_counts::word.eq(query.word.text.msg.to_string()))
                    .get_results(&mut *self.connection.lock().await)?
            };

            records
                .into_iter()
                .map(|record| {
                    Ok(GetWordsCountsOutput {
                        word: GetWordKeyHash {
                            key: WordKeyHash {
                                namespace: record.namespace.parse()?,
                                text: TextHash {
                                    lang: record.lang.parse()?,
                                    msg: record.word.parse()?,
                                },
                            },
                            kind: record.kind.parse()?,
                        },
                        count: record.count.try_into()?,
                    })
                })
                .collect()
        }
    }

    async fn put_word_unchecked(
        &self,
        parent: &Hash,
        word: &GuaranteeSigned<WordHash>,
    ) -> Result<()> {
        let word = self.ipiis.sign_as_guarantor(*word)?;

        let record = crate::models::words::NewWord {
            nonce: word.nonce.0 .0,
            guarantee: word.guarantee.account.to_string(),
            guarantor: word.guarantor.account.to_string(),
            guarantee_signature: word.guarantee.signature.to_string(),
            guarantor_signature: word.guarantor.signature.to_string(),
            created_date: word.created_date.naive_utc(),
            expiration_date: word.expiration_date.map(|e| e.naive_utc()),
            namespace: word.data.key.namespace.to_string(),
            parent: parent.to_string(),
            lang: word.data.key.text.lang.to_string(),
            word: word.data.key.text.msg.to_string(),
            kind: word.data.kind.to_string(),
            relpath: word.data.relpath,
            path: word.data.path.value.to_string(),
            len: word.data.path.len.try_into()?,
        };

        self.connection
            .lock()
            .await
            .transaction::<(), ::diesel::result::Error, _>(|conn| {
                // insert the word record
                ::diesel::insert_into(crate::schema::words::table)
                    .values(&record)
                    .execute(conn)?;

                // check whether word exists
                match crate::schema::words_counts::table
                    .filter(crate::schema::words_counts::namespace.eq(&record.namespace))
                    .filter(crate::schema::words_counts::kind.eq(&record.kind))
                    .filter(crate::schema::words_counts::parent.eq(&record.parent))
                    .filter(crate::schema::words_counts::lang.eq(&record.lang))
                    .filter(crate::schema::words_counts::word.eq(&record.word))
                    .get_results::<crate::models::words::WordCount>(conn)?
                    .pop()
                {
                    // old word => append the count
                    Some(word_count) => ::diesel::update(crate::schema::words_counts::table)
                        .filter(crate::schema::words_counts::id.eq(word_count.id))
                        .set(crate::schema::words_counts::count.eq(word_count.count + 1))
                        .execute(conn)?,
                    // new word => insert the word record
                    None => {
                        let word_record = crate::models::words::NewWordCount {
                            namespace: record.namespace.clone(),
                            kind: record.kind.clone(),
                            parent: record.parent.clone(),
                            lang: record.lang.clone(),
                            word: record.word.clone(),
                            count: 1,
                        };

                        ::diesel::insert_into(crate::schema::words_counts::table)
                            .values(&word_record)
                            .execute(conn)?
                    }
                };

                // check whether word of guarantee exists
                match crate::schema::words_counts_guarantees::table
                    .filter(crate::schema::words_counts_guarantees::guarantee.eq(&record.guarantee))
                    .filter(crate::schema::words_counts_guarantees::kind.eq(&record.kind))
                    .filter(crate::schema::words_counts_guarantees::parent.eq(&record.parent))
                    .filter(crate::schema::words_counts_guarantees::lang.eq(&record.lang))
                    .filter(crate::schema::words_counts_guarantees::word.eq(&record.word))
                    .get_results::<crate::models::words::WordCountGuarantee>(conn)?
                    .pop()
                {
                    // old word => append the count
                    Some(word_count_guarantee) => {
                        ::diesel::update(crate::schema::words_counts_guarantees::table)
                            .filter(
                                crate::schema::words_counts_guarantees::id
                                    .eq(word_count_guarantee.id),
                            )
                            .set(
                                crate::schema::words_counts_guarantees::count
                                    .eq(word_count_guarantee.count + 1),
                            )
                            .execute(conn)?
                    }
                    // new word => insert the word record
                    None => {
                        let word_record = crate::models::words::NewWordCountGuarantee {
                            guarantee: record.guarantee.clone(),
                            namespace: record.namespace.clone(),
                            kind: record.kind.clone(),
                            parent: record.parent.clone(),
                            lang: record.lang.clone(),
                            word: record.word.clone(),
                            count: 1,
                        };

                        ::diesel::insert_into(crate::schema::words_counts_guarantees::table)
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
    IpiisClient: Ipiis + Send + Sync,
{
    pub async fn delete_guarantee_unchecked(&self, guarantee: &AccountRef) -> Result<()> {
        ::diesel::delete(crate::schema::accounts_guarantees::table)
            .filter(crate::schema::accounts_guarantees::guarantee.eq(guarantee.to_string()))
            .execute(&mut *self.connection.lock().await)
            .map(|_| ())
            .map_err(Into::into)
    }

    pub async fn delete_dyn_path_all_unchecked(&self, namespace: &Hash) -> Result<()> {
        ::diesel::delete(crate::schema::dyn_paths::table)
            .filter(crate::schema::dyn_paths::namespace.eq(namespace.to_string()))
            .execute(&mut *self.connection.lock().await)
            .map(|_| ())
            .map_err(Into::into)
    }

    pub async fn delete_word_all_unchecked(&self, namespace: &Hash) -> Result<()> {
        self.connection
            .lock()
            .await
            .transaction::<(), ::diesel::result::Error, _>(|conn| {
                ::diesel::delete(crate::schema::words::table)
                    .filter(crate::schema::words::namespace.eq(namespace.to_string()))
                    .execute(conn)
                    .map(|_| ())?;

                ::diesel::delete(crate::schema::words_counts::table)
                    .filter(crate::schema::words_counts::namespace.eq(namespace.to_string()))
                    .execute(conn)
                    .map(|_| ())?;

                ::diesel::delete(crate::schema::words_counts_guarantees::table)
                    .filter(
                        crate::schema::words_counts_guarantees::namespace.eq(namespace.to_string()),
                    )
                    .execute(conn)
                    .map(|_| ())?;

                Ok(())
            })
            .map_err(Into::into)
    }
}
