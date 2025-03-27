//! SP1 proof program for Battleship game verification.
#![no_main]
sp1_zkvm::entrypoint!(main);

use alloy_sol_types::{SolType, private::FixedBytes};
use battleship_proof_lib::{verify_game_result, PublicValuesStruct};

pub fn main() {
    // Read input data
    let username_hash = sp1_zkvm::io::read::<[u8; 32]>();
    let ships_sunk = sp1_zkvm::io::read::<u32>();
    let total_shots = sp1_zkvm::io::read::<u32>();
    let hit_percentage = sp1_zkvm::io::read::<u32>();
    let timestamp = sp1_zkvm::io::read::<u32>();
    let winner = sp1_zkvm::io::read::<bool>();
    
    // Verification process
    let is_valid = verify_game_result(
        ships_sunk, 
        total_shots, 
        hit_percentage, 
        winner
    );
    
    // Create public values
    let public_values = PublicValuesStruct {
        usernameHash: FixedBytes(username_hash),
        shipsSunk: ships_sunk,
        totalShots: total_shots,
        hitPercentage: hit_percentage,
        timestamp: timestamp,
        verified: if is_valid { 1 } else { 0 },
    };
    
    // Debug outputs
    println!("Battleship Game Verification:");
    println!("Username Hash: 0x{}", hex::encode(username_hash));
    println!("Ships Sunk: {}", ships_sunk);
    println!("Total Shots: {}", total_shots);
    println!("Hit Percentage: {}%", hit_percentage);
    println!("Timestamp: {}", timestamp);
    println!("Winner: {}", winner);
    println!("Verification Result: {}", if is_valid { "VERIFIED" } else { "FAILED" });
    
    // Encode results and provide as output
    let bytes = PublicValuesStruct::abi_encode(&public_values);
    sp1_zkvm::io::commit_slice(&bytes);
}
