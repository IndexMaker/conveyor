use alloy::sol;

sol! {
    #[sol(rpc)]
    interface IWorksman  {
        function setVaultPrototype(address vault_implementation) external;

        function buildVault() external returns (address);
    }
}