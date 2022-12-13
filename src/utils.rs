use anyhow::Result;
use regex::Regex;
use starknet::{
    core::types::FieldElement,
    providers::jsonrpc::models::{BlockId, BlockTag},
};

pub fn parse_block_id(id: &str) -> Result<BlockId> {
    let regex_block_number = Regex::new("^[0-9]{1,}$").unwrap();

    if id == "latest" {
        Ok(BlockId::Tag(BlockTag::Latest))
    } else if id == "pending" {
        Ok(BlockId::Tag(BlockTag::Pending))
    } else if regex_block_number.is_match(id) {
        Ok(BlockId::Number(id.parse::<u64>()?))
    } else {
        Ok(BlockId::Hash(FieldElement::from_hex_be(id)?))
    }
}

pub fn parse_felt_value(felt: &str) -> Result<FieldElement> {
    let regex_dec_number = Regex::new("^[0-9]{1,}$").unwrap();

    if regex_dec_number.is_match(felt) {
        Ok(FieldElement::from_dec_str(felt)?)
    } else {
        Ok(FieldElement::from_hex_be(felt)?)
    }
}
