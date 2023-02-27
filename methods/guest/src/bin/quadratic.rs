#![no_main]
#![no_std]

use ethabi::ethereum_types::U256;
use ethabi::{ParamType, Token};
use merkletree::hash::Algorithm;
use merkletree::merkle::MerkleTree;
use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);

fn quadratic(donations: Vec<Vec<U256>>, matching_amount: U256) -> U256 {
    // for each grant define x, for each donation add donation**2 to x
    let mut x = vec![U256::zero(); donations.len()];
    let mut y = vec![U256::zero(); donations.len()];

    let mut cumulative_quadratic = U256::zero();

    let mut receive = vec![U256::zero(); donations.len()];

    for (i, grant) in donations.iter().enumerate() {
        for donation in grant {
            x[i] += donation * donation;
            y[i] += donation;
        }
        x[i] = x[i].pow(U256::from(2));

        cumulative_quadratic += x[i];
    }

    for (i, grant) in donations.iter().enumerate() {
        let mut receive_from_match = x[i] * matching_amount / cumulative_quadratic;
        let mut receive_from_grant = y[i] + receive_from_match;

        receive[i] = receive_from_grant;
    }

    // create merkle tree of receive, where leaves are keccak256(i, receive[i]) using merkletree library
    let mut leaves = vec![];
    for (i, r) in receive.iter().enumerate() {
        let mut leaf = vec![];
        leaf.extend_from_slice(&i.to_be_bytes());
        leaf.extend_from_slice(&r.to_be_bytes());
        leaves.push(leaf);
    }

    let tree = MerkleTree::<sha3::Keccak256, _>::new(&leaves);

    // return root of merkle tree
    let root = tree.root();

    let mut root_bytes = [0u8; 32];
    root_bytes.copy_from_slice(&root);

    U256::from(root_bytes)
}

pub fn main() {
    // NOTE: Currently env::read_slice requires a length argument. Bonsai passes the length at the
    // start of the input. https://github.com/risc0/risc0/issues/402
    let input_len = env::read_slice::<u32>(1)[0] as usize;

    // Decode the input.
    // input is a tuple of ([][]uint256, uint256)
    let input = env::read_slice::<u8>(input_len);
    let (donations, matchingAmount) = ethabi::decode(
        &[
            ParamType::Array(Box::new(ParamType::Array(Box::new(ParamType::Uint(256))))),
            ParamType::Uint(256),
        ],
        &input,
    )
    .unwrap();

    // Run the computation.
    let result = quadratic(donations, matchingAmount);

    // Commit the journal that will be decoded in the application contract.
    env::commit_slice(&ethabi::encode(&[Token::Uint(result)]));
}
