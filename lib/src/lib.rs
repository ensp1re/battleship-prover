use alloy_sol_types::sol;

sol! {
    struct PublicValuesStruct {
        bytes32 usernameHash;    // Hash of the username
        uint32 shipsSunk;        // Number of ships sunk
        uint32 totalShots;       // Total shots fired
        uint32 hitPercentage;    // Percentage of successful hits
        uint32 timestamp;        // UNIX timestamp of verification
        uint32 verified;         // Verification status (1=verified, 0=failed)
    }
}

/// Verify the game result based on specific criteria
pub fn verify_game_result(
    ships_sunk: u32, 
    total_shots: u32, 
    hit_percentage: u32, 
    winner: bool
) -> bool {
    // Validate game parameters
    let valid_ships = ships_sunk <= 5;  // Max 5 ships in standard game
    let valid_shots = total_shots >= 10 && total_shots <= 100;  // Reasonable shot range
    let valid_hit_percentage = hit_percentage <= 100;  // Valid percentage
    
    // Additional logic to validate game outcome
    let strategic_validity = if winner {
        // If winner, require at least some performance metrics
        ships_sunk > 0 && hit_percentage > 30
    } else {
        // If not winner, allow lower performance
        true
    };
    
    valid_ships && valid_shots && valid_hit_percentage && strategic_validity
}
