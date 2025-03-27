use sp1_sdk::{include_elf, HashableKey, ProverClient};

/// RISC-V ELF file for the Battleship proof program.
pub const BATTLESHIP_PROOF_ELF: &[u8] = include_elf!("battleship_proof_program");

fn main() {
    // Setup prover client
    let client = ProverClient::from_env();
    
    // Get verification key for the program
    let (_, vk) = client.setup(BATTLESHIP_PROOF_ELF);
    
    // Print verification key
    println!("Battleship Game Proof Program VKey: {}", vk.bytes32());
}
