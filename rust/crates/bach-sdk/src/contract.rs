//! Contract interaction helpers

use bach_primitives::Address;
use bytes::Bytes;

use crate::abi::{decode, encode_function_call, function_selector, ParamType, Token};
use crate::SdkError;

/// Contract helper for encoding/decoding function calls
#[derive(Debug, Clone)]
pub struct Contract {
    /// Contract address
    address: Address,
    /// Function definitions
    functions: Vec<FunctionDef>,
}

/// Function definition
#[derive(Debug, Clone)]
pub struct FunctionDef {
    /// Function name
    pub name: String,
    /// Function signature (e.g., "transfer(address,uint256)")
    pub signature: String,
    /// Function selector (4 bytes)
    pub selector: [u8; 4],
    /// Input parameter types
    pub inputs: Vec<ParamType>,
    /// Output parameter types
    pub outputs: Vec<ParamType>,
}

impl FunctionDef {
    /// Create a new function definition
    pub fn new(
        name: impl Into<String>,
        signature: impl Into<String>,
        inputs: Vec<ParamType>,
        outputs: Vec<ParamType>,
    ) -> Self {
        let signature = signature.into();
        let selector = function_selector(&signature);
        Self {
            name: name.into(),
            signature,
            selector,
            inputs,
            outputs,
        }
    }
}

impl Contract {
    /// Create a new contract helper
    pub fn new(address: Address) -> Self {
        Self {
            address,
            functions: Vec::new(),
        }
    }

    /// Get the contract address
    pub fn address(&self) -> &Address {
        &self.address
    }

    /// Add a function definition
    pub fn add_function(&mut self, function: FunctionDef) {
        self.functions.push(function);
    }

    /// Add a function with builder pattern
    pub fn with_function(mut self, function: FunctionDef) -> Self {
        self.functions.push(function);
        self
    }

    /// Get a function by name
    pub fn function(&self, name: &str) -> Option<&FunctionDef> {
        self.functions.iter().find(|f| f.name == name)
    }

    /// Encode a function call
    pub fn encode_call(&self, function_name: &str, args: &[Token]) -> Result<Bytes, SdkError> {
        let function = self
            .function(function_name)
            .ok_or_else(|| SdkError::AbiEncode(format!("Unknown function: {}", function_name)))?;

        if args.len() != function.inputs.len() {
            return Err(SdkError::AbiEncode(format!(
                "Expected {} arguments, got {}",
                function.inputs.len(),
                args.len()
            )));
        }

        let data = encode_function_call(function.selector, args);
        Ok(Bytes::from(data))
    }

    /// Decode function output
    pub fn decode_output(&self, function_name: &str, data: &[u8]) -> Result<Vec<Token>, SdkError> {
        let function = self
            .function(function_name)
            .ok_or_else(|| SdkError::AbiDecode(format!("Unknown function: {}", function_name)))?;

        decode(&function.outputs, data)
    }
}

/// Builder for creating common contract interfaces
pub struct ContractBuilder {
    address: Address,
    functions: Vec<FunctionDef>,
}

impl ContractBuilder {
    /// Create a new contract builder
    pub fn new(address: Address) -> Self {
        Self {
            address,
            functions: Vec::new(),
        }
    }

    /// Add a function
    pub fn function(
        mut self,
        name: &str,
        signature: &str,
        inputs: Vec<ParamType>,
        outputs: Vec<ParamType>,
    ) -> Self {
        self.functions.push(FunctionDef::new(name, signature, inputs, outputs));
        self
    }

    /// Build the contract
    pub fn build(self) -> Contract {
        Contract {
            address: self.address,
            functions: self.functions,
        }
    }
}

/// Create an ERC20 contract helper
pub fn erc20(address: Address) -> Contract {
    ContractBuilder::new(address)
        .function(
            "name",
            "name()",
            vec![],
            vec![ParamType::String],
        )
        .function(
            "symbol",
            "symbol()",
            vec![],
            vec![ParamType::String],
        )
        .function(
            "decimals",
            "decimals()",
            vec![],
            vec![ParamType::Uint(8)],
        )
        .function(
            "totalSupply",
            "totalSupply()",
            vec![],
            vec![ParamType::Uint(256)],
        )
        .function(
            "balanceOf",
            "balanceOf(address)",
            vec![ParamType::Address],
            vec![ParamType::Uint(256)],
        )
        .function(
            "transfer",
            "transfer(address,uint256)",
            vec![ParamType::Address, ParamType::Uint(256)],
            vec![ParamType::Bool],
        )
        .function(
            "approve",
            "approve(address,uint256)",
            vec![ParamType::Address, ParamType::Uint(256)],
            vec![ParamType::Bool],
        )
        .function(
            "allowance",
            "allowance(address,address)",
            vec![ParamType::Address, ParamType::Address],
            vec![ParamType::Uint(256)],
        )
        .function(
            "transferFrom",
            "transferFrom(address,address,uint256)",
            vec![ParamType::Address, ParamType::Address, ParamType::Uint(256)],
            vec![ParamType::Bool],
        )
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bach_primitives::U256;

    #[test]
    fn test_contract_encode_call() {
        let addr = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d").unwrap();
        let contract = erc20(addr);

        let to = Address::from_hex("0x1234567890123456789012345678901234567890").unwrap();
        let amount = U256::from(1000);

        let data = contract
            .encode_call("transfer", &[Token::Address(to), Token::Uint(amount)])
            .unwrap();

        // Should start with transfer selector
        assert_eq!(&data[..4], &[0xa9, 0x05, 0x9c, 0xbb]);
        assert_eq!(data.len(), 68); // 4 + 32 + 32
    }

    #[test]
    fn test_contract_encode_balance_of() {
        let addr = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d").unwrap();
        let contract = erc20(addr);

        let owner = Address::from_hex("0x1234567890123456789012345678901234567890").unwrap();

        let data = contract
            .encode_call("balanceOf", &[Token::Address(owner)])
            .unwrap();

        // Should start with balanceOf selector
        assert_eq!(&data[..4], &[0x70, 0xa0, 0x82, 0x31]);
        assert_eq!(data.len(), 36); // 4 + 32
    }

    #[test]
    fn test_contract_decode_output() {
        let addr = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d").unwrap();
        let contract = erc20(addr);

        let mut data = [0u8; 32];
        data[31] = 100;

        let tokens = contract.decode_output("balanceOf", &data).unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Uint(U256::from(100)));
    }

    #[test]
    fn test_contract_unknown_function() {
        let addr = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d").unwrap();
        let contract = erc20(addr);

        let result = contract.encode_call("unknown", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_contract_wrong_arg_count() {
        let addr = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d").unwrap();
        let contract = erc20(addr);

        let result = contract.encode_call("transfer", &[Token::Address(Address::ZERO)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_erc20_helper() {
        let addr = Address::ZERO;
        let contract = erc20(addr);

        assert!(contract.function("name").is_some());
        assert!(contract.function("symbol").is_some());
        assert!(contract.function("decimals").is_some());
        assert!(contract.function("totalSupply").is_some());
        assert!(contract.function("balanceOf").is_some());
        assert!(contract.function("transfer").is_some());
        assert!(contract.function("approve").is_some());
        assert!(contract.function("allowance").is_some());
        assert!(contract.function("transferFrom").is_some());
    }

    #[test]
    fn test_function_def_selector() {
        let func = FunctionDef::new(
            "transfer",
            "transfer(address,uint256)",
            vec![ParamType::Address, ParamType::Uint(256)],
            vec![ParamType::Bool],
        );

        assert_eq!(func.selector, [0xa9, 0x05, 0x9c, 0xbb]);
    }

    #[test]
    fn test_contract_builder() {
        let addr = Address::ZERO;
        let contract = ContractBuilder::new(addr)
            .function(
                "myFunction",
                "myFunction(uint256)",
                vec![ParamType::Uint(256)],
                vec![ParamType::Bool],
            )
            .build();

        assert!(contract.function("myFunction").is_some());
        assert_eq!(contract.address(), &Address::ZERO);
    }
}
