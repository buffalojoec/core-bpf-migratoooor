use {solana_client::nonblocking::rpc_client::RpcClient, solana_sdk::signature::Keypair};

pub struct TestContext<'a> {
    pub rpc_client: &'a RpcClient,
    pub payer: &'a Keypair,
}
impl TestContext<'_> {
    pub fn new<'a>(rpc_client: &'a RpcClient, payer: &'a Keypair) -> TestContext<'a> {
        TestContext { rpc_client, payer }
    }
}
