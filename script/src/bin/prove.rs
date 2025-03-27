use alloy_sol_types::SolType;
use clap::Parser;
use battleship_proof_lib::PublicValuesStruct;
use sp1_sdk::{include_elf, ProverClient, SP1Stdin};
use std::time::{SystemTime, UNIX_EPOCH};

/// RISC-V ELF file for the Battleship proof program.
pub const BATTLESHIP_PROOF_ELF: &[u8] = include_elf!("battleship_proof_program");

/// Command line arguments
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long)]
    execute: bool,
    
    #[clap(long)]
    prove: bool,
    
    #[clap(long, default_value = "anonymous")]
    username: String,
    
    #[clap(long, default_value = "0")]
    ships_sunk: u32,
    
    #[clap(long, default_value = "0")]
    total_shots: u32,
    
    #[clap(long, default_value = "0")]
    hit_percentage: u32,
    
    #[clap(long)]
    winner: Option<bool>,
    
    #[clap(long, default_value = "0000000000000000000000000000000000000000000000000000000000000000")]
    user_hash: String,
}

fn main() {
    // Setup logger
    sp1_sdk::utils::setup_logger();
    dotenv::dotenv().ok();
    
    // Parse command line arguments
    let args = Args::parse();
    
    if args.execute == args.prove {
        eprintln!("Error: You must specify either --execute or --prove");
        std::process::exit(1);
    }
    
    // Setup prover client
    let client = ProverClient::from_env();
    
    // Parse username hash
    let user_hash_bytes: [u8; 32] = if args.user_hash.len() == 64 {
        hex::decode(&args.user_hash)
            .expect("Invalid user hash hex string")
            .try_into()
            .expect("Invalid user hash length")
    } else {
        // Generate hash from username if custom hash not provided
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(args.username.as_bytes());
        hasher.finalize().into()
    };
    
    // Get current timestamp
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as u32;
    
    // Prepare inputs
    let mut stdin = SP1Stdin::new();
    stdin.write(&user_hash_bytes);
    stdin.write(&args.ships_sunk);
    stdin.write(&args.total_shots);
    stdin.write(&args.hit_percentage);
    stdin.write(&timestamp);
    stdin.write(&args.winner);
    
    
    if args.execute {
        // Run program without generating proof
        let (output, report) = client.execute(BATTLESHIP_PROOF_ELF, &stdin).run().unwrap();
        println!("Program executed successfully.");
        
        // Read output
        let decoded = PublicValuesStruct::abi_decode(output.as_slice(), true).unwrap();
        let PublicValuesStruct { 
            usernameHash, 
            shipsSunk, 
            totalShots, 
            hitPercentage, 
            timestamp, 
            verified 
        } = decoded;
        
        println!("Battleship Game Verification Result:");
        println!("Username Hash: 0x{}", hex::encode(usernameHash.0));
        println!("Ships Sunk: {}", shipsSunk);
        println!("Total Shots: {}", totalShots);
        println!("Hit Percentage: {}%", hitPercentage);
        println!("Timestamp: {}", timestamp);
        println!("Verification Status: {}", if verified == 1 { "VERIFIED" } else { "FAILED" });
        
        // Log executed instruction count
        println!("Number of instructions executed: {}", report.total_instruction_count());
    } else {
        // Setup program for proof generation
        let (pk, vk) = client.setup(BATTLESHIP_PROOF_ELF);
        
        // Generate proof
        let proof = client
            .prove(&pk, &stdin)
            .run()
            .expect("proof generation failed");
        
        println!("Proof successfully generated!");
        
        // Verify proof
        client.verify(&proof, &vk).expect("proof verification failed");
        println!("Proof successfully verified!");
        
        // Save proof to disk
        let proof_path = format!(
            "battleship_proof_{}_{}_{}_{}_{}.bin", 
            args.username, 
            args.ships_sunk, 
            args.total_shots, 
            args.hit_percentage, 
            timestamp
        );
        proof.save(&proof_path).expect("failed to save proof");
        println!("Proof saved to file: {}", proof_path);
    }
}
