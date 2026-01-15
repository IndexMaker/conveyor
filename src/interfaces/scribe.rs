use alloy::sol;

sol! {
    #[sol(rpc)]
    interface IScribe  {
        function verifySignature(bytes calldata data) external returns (bool);
    }
}