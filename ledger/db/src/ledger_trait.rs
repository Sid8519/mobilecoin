// Copyright (c) 2018-2021 The MobileCoin Foundation

use crate::{mint_config_store::ActiveMintConfig, Error};
use mc_common::Hash;
use mc_crypto_keys::CompressedRistrettoPublic;
use mc_transaction_core::{
    mint::MintTx,
    ring_signature::KeyImage,
    tx::{TxOut, TxOutMembershipElement, TxOutMembershipProof},
    Block, BlockContents, BlockData, BlockIndex, BlockSignature, TokenId,
};
use mockall::*;

#[automock]
pub trait Ledger: Send {
    /// Appends a block along with transactions.
    fn append_block(
        &mut self,
        block: &Block,
        block_contents: &BlockContents,
        signature: Option<BlockSignature>,
    ) -> Result<(), Error>;

    /// Get the total number of blocks in the ledger.
    fn num_blocks(&self) -> Result<u64, Error>;

    /// Gets a Block by its index in the blockchain.
    fn get_block(&self, block_number: BlockIndex) -> Result<Block, Error>;

    /// Get the contents of a block.
    fn get_block_contents(&self, block_number: BlockIndex) -> Result<BlockContents, Error>;

    /// Gets a block signature by its index in the blockchain.
    fn get_block_signature(&self, block_number: BlockIndex) -> Result<BlockSignature, Error>;

    /// Gets a block and all of its associated data by its index in the
    /// blockchain.
    fn get_block_data(&self, block_number: BlockIndex) -> Result<BlockData, Error>;

    /// Gets block index by a TxOut global index.
    fn get_block_index_by_tx_out_index(&self, tx_out_index: u64) -> Result<BlockIndex, Error>;

    /// Get the total number of TxOuts in the ledger.
    fn num_txos(&self) -> Result<u64, Error>;

    /// Returns the index of the TxOut with the given hash.
    fn get_tx_out_index_by_hash(&self, tx_out_hash: &Hash) -> Result<u64, Error>;

    /// Returns the index of the TxOut with the given public key.
    fn get_tx_out_index_by_public_key(
        &self,
        tx_out_public_key: &CompressedRistrettoPublic,
    ) -> Result<u64, Error>;

    /// Gets a TxOut by its index in the ledger.
    fn get_tx_out_by_index(&self, index: u64) -> Result<TxOut, Error>;

    /// Gets a proof of memberships for TxOuts with indexes `indexes`.
    fn get_tx_out_proof_of_memberships(
        &self,
        indexes: &[u64],
    ) -> Result<Vec<TxOutMembershipProof>, Error>;

    /// Returns true if the Ledger contains the given TxOut public key.
    fn contains_tx_out_public_key(
        &self,
        public_key: &CompressedRistrettoPublic,
    ) -> Result<bool, Error>;

    /// Returns true if the Ledger contains the given key image.
    fn contains_key_image(&self, key_image: &KeyImage) -> Result<bool, Error> {
        self.check_key_image(key_image).map(|x| x.is_some())
    }

    /// Checks if the ledger contains a given key image.
    /// If so, returns the index of the block in which it entered the ledger.
    /// Ok(None) is returned when the key image is not in the ledger.
    fn check_key_image(&self, key_image: &KeyImage) -> Result<Option<BlockIndex>, Error>;

    /// Gets the key images used by transactions in a single block.
    fn get_key_images_by_block(&self, block_number: BlockIndex) -> Result<Vec<KeyImage>, Error>;

    /// Get the tx out root membership element from the tx out Merkle Tree.
    fn get_root_tx_out_membership_element(&self) -> Result<TxOutMembershipElement, Error>;

    /// Get the latest block header, if any. NotFound if the ledger-db is empty.
    fn get_latest_block(&self) -> Result<Block, Error> {
        let num_blocks = self.num_blocks()?;
        if num_blocks == 0 {
            return Err(Error::NotFound);
        }
        self.get_block(num_blocks - 1)
    }

    /// Get active mint configurations for a given token id.
    /// Returns an empty array if no mint configurations are active for the
    /// given token id.
    fn get_active_mint_configs(&self, token_id: TokenId) -> Result<Vec<ActiveMintConfig>, Error>;

    /// Checks if the ledger contains a given MintConfigTx nonce.
    /// If so, returns the index of the block in which it entered the ledger.
    /// Ok(None) is returned when the nonce is not in the ledger.
    fn check_mint_config_tx_nonce(&self, nonce: &[u8]) -> Result<Option<BlockIndex>, Error>;

    /// Checks if the ledger contains a given MintTx nonce.
    /// If so, returns the index of the block in which it entered the ledger.
    /// Ok(None) is returned when the nonce is not in the ledger.
    fn check_mint_tx_nonce(&self, nonce: &[u8]) -> Result<Option<BlockIndex>, Error>;

    /// Attempt to get an active mint configuration that is able to verify and
    /// accommodate a given MintTx.
    fn get_active_mint_config_for_mint_tx(
        &self,
        mint_tx: &MintTx,
    ) -> Result<ActiveMintConfig, Error>;
}
