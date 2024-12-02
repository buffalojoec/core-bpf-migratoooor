//! RPC Client ops.

use {
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
    solana_sdk::{bpf_loader_upgradeable::UpgradeableLoaderState, pubkey::Pubkey},
};

pub struct CbmRpcClient {
    rpc_client: RpcClient,
}

impl CbmRpcClient {
    pub fn new(url: &str) -> Self {
        Self {
            rpc_client: RpcClient::new(url.to_string()),
        }
    }

    pub async fn clone_elf_from_buffer_account(&self, buffer_address: &Pubkey) -> Vec<u8> {
        let account = self
            .rpc_client
            .get_account(buffer_address)
            .await
            .expect("Account not found");
        if account.data.len() < UpgradeableLoaderState::size_of_buffer_metadata() {
            panic!("Buffer account is too small");
        }
        account.data[UpgradeableLoaderState::size_of_buffer_metadata()..].to_vec()
    }
}
