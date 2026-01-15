use alloy::sol;

sol! {
    #[sol(rpc)]
    interface IClerk  {
        function updateRecords(bytes calldata code, uint128 num_registry) external;
    }
}