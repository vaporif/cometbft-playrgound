use std::{fmt::Debug, mem::ManuallyDrop, path::Path};

pub const STATE_CF: &str = "state";

use borsh::{BorshDeserialize, BorshSerialize};
use eyre::Context;
use rocksdb::{
    BlockBasedOptions, ColumnFamily, ColumnFamilyDescriptor, DBCompactionStyle, DBCompressionType,
    Options, WriteBatch,
};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[allow(missing_docs)]
#[derive(Error, Debug)]
pub enum Error {
    #[error("DB error: {0}")]
    DBError(String),
}

#[derive(Debug)]
pub struct RocksDB {
    inner: ManuallyDrop<rocksdb::DB>,
    invalid_handle: bool,
    read_only: bool,
}

#[derive(Default)]
pub struct RocksDBWriteBatch(WriteBatch);

impl DBWriteBatch for RocksDBWriteBatch {}

impl RocksDB {
    pub fn open(
        path: impl AsRef<Path>,
        cache: Option<rocksdb::Cache>,
        read_only: bool,
    ) -> eyre::Result<Self> {
        //let logical_cores = num_cpus::get();
        //let compaction_threads = i32::try_from(num_of_threads(
        //    ENV_VAR_ROCKSDB_COMPACTION_THREADS,
        //    logical_cores / 4,
        //))?;
        //tracing::info!(
        //    "Using {} compactions threads for RocksDB.",
        //    compaction_threads
        //);

        let mut cfs = Vec::new();
        let mut db_opts = Options::default();

        let mut table_opts = BlockBasedOptions::default();
        //db_opts.increase_parallelism(compaction_threads);

        let mut diffs_cf_opts = Options::default();
        diffs_cf_opts.set_compression_type(DBCompressionType::Zstd);
        diffs_cf_opts.set_compression_options(0, 0, 0, 1024 * 1024);
        diffs_cf_opts.set_compaction_style(DBCompactionStyle::Universal);
        diffs_cf_opts.set_block_based_table_factory(&table_opts);
        cfs.push(ColumnFamilyDescriptor::new(STATE_CF, diffs_cf_opts));
        db_opts.set_bytes_per_sync(1048576);

        db_opts.create_missing_column_families(true);
        db_opts.create_if_missing(true);
        db_opts.set_atomic_flush(true);

        if let Some(cache) = cache {
            table_opts.set_block_cache(&cache);
        }

        Ok(if read_only {
            RocksDB {
                inner: ManuallyDrop::new(
                    rocksdb::DB::open_cf_descriptors_read_only(&db_opts, path, cfs, false)
                        .wrap_err("db error")?,
                ),
                invalid_handle: false,
                read_only: true,
            }
        } else {
            RocksDB {
                inner: ManuallyDrop::new(
                    rocksdb::DB::open_cf_descriptors(&db_opts, path, cfs).wrap_err("db error")?,
                ),
                invalid_handle: false,
                read_only: false,
            }
        })
    }

    fn read_value<T: BorshDeserialize>(
        &self,
        cf: &ColumnFamily,
        key: impl AsRef<str>,
    ) -> Result<Option<T>> {
        self.inner
            .get_cf(cf, key.as_ref())?
            .map(|v| borsh::de::from_slice(&v).context("decode"))
            .transpose()
    }

    fn read_value_bytes(&self, cf: &ColumnFamily, key: impl AsRef<str>) -> Result<Option<Vec<u8>>> {
        self.inner
            .get_cf(cf, key.as_ref())
            .map_err(|e| Error::DBError(e.into_string()))
    }
}

impl Drop for RocksDB {
    fn drop(&mut self) {
        if self.invalid_handle {
            return;
        }
        if !self.read_only {
            self.flush(true).expect("flush failed");
        }
        unsafe { ManuallyDrop::drop(&mut self.inner) }
    }
}

pub trait DB: Debug {
    type Cache;

    type WriteBatch: DBWriteBatch;

    fn open(db_path: impl AsRef<std::path::Path>, cache: Option<&Self::Cache>) -> Self;

    fn path(&self) -> Option<&std::path::Path> {
        None
    }

    fn read_val(&self, key: String) -> Result<Option<Vec<u8>>>;

    fn batch() -> Self::WriteBatch;

    fn exec_batch(&self, batch: Self::WriteBatch) -> Result<()>;

    fn flush(&self, wait: bool) -> Result<()>;

    fn read_last_block(&self) -> Result<Option<Block>>;
}

pub trait DBWriteBatch {}

impl DB for RocksDB {
    type Cache = rocksdb::Cache;

    type WriteBatch = RocksDBWriteBatch;

    fn open(db_path: impl AsRef<std::path::Path>, cache: Option<&Self::Cache>) -> Self {
        todo!()
    }

    fn read_val(&self, key: String) -> Result<Option<Vec<u8>>> {
        todo!()
    }

    fn batch() -> Self::WriteBatch {
        todo!()
    }

    fn exec_batch(&self, batch: Self::WriteBatch) -> Result<()> {
        todo!()
    }

    fn flush(&self, wait: bool) -> Result<()> {
        todo!()
    }

    fn read_last_block(&self) -> Result<Option<Block>> {
        todo!()
    }
}
