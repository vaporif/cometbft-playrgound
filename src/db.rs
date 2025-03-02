use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[allow(missing_docs)]
#[derive(Error, Debug)]
pub enum Error {
    #[error("DB error: {0}")]
    DBError(String),
}

pub struct BlockStateRead {
    /// Height of the block
    pub height: BlockHeight,
    /// Time of the block
    pub time: DateTimeUtc,
    /// Epoch of the block
    /// Minimum block height at which the next epoch may start
    pub next_epoch_min_start_height: BlockHeight,
    /// Minimum block time at which the next epoch may start
    pub next_epoch_min_start_time: DateTimeUtc,
    /// Update epoch delay
    pub update_epoch_blocks_delay: Option<u32>,
    /// Established address generator
    pub address_gen: EstablishedAddressGen,
    /// Results of applying transactions
    pub results: BlockResults,
    /// The conversion state
    pub conversion_state: ConversionState,
    /// The latest block height on Ethereum processed, if
    /// the bridge is enabled.
    pub ethereum_height: Option<ethereum_structs::BlockHeight>,
    /// The queue of Ethereum events to be processed in order.
    pub eth_events_queue: EthEventsQueue,
    /// Structure holding data that needs to be added to the merkle tree
    pub commit_only_data: CommitOnlyData,
}

/// The block's state to write into the database.
pub struct BlockStateWrite<'a> {
    /// Merkle tree stores
    pub merkle_tree_stores: MerkleTreeStoresWrite<'a>,
    /// Header of the block
    pub header: Option<&'a BlockHeader>,
    /// Height of the block
    pub height: BlockHeight,
    /// Time of the block
    pub time: DateTimeUtc,
    /// Epoch of the block
    pub epoch: Epoch,
    /// Predecessor block epochs
    pub pred_epochs: &'a Epochs,
    /// Minimum block height at which the next epoch may start
    pub next_epoch_min_start_height: BlockHeight,
    /// Minimum block time at which the next epoch may start
    pub next_epoch_min_start_time: DateTimeUtc,
    /// Update epoch delay
    pub update_epoch_blocks_delay: Option<u32>,
    /// Established address generator
    pub address_gen: &'a EstablishedAddressGen,
    /// Results of applying transactions
    pub results: &'a BlockResults,
    /// The conversion state
    pub conversion_state: &'a ConversionState,
    /// The latest block height on Ethereum processed, if
    /// the bridge is enabled.
    pub ethereum_height: Option<&'a ethereum_structs::BlockHeight>,
    /// The queue of Ethereum events to be processed in order.
    pub eth_events_queue: &'a EthEventsQueue,
    /// Structure holding data that needs to be added to the merkle tree
    pub commit_only_data: &'a CommitOnlyData,
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
