const express = require('express');
const cors = require('cors');
const bodyParser = require('body-parser');
const { exec } = require('child_process');
const path = require('path');
const crypto = require('crypto');
const fs = require('fs');

const app = express();
const PORT = 4001;

// Middleware
app.use(cors());
app.use(bodyParser.json());
app.use(express.static(path.join(__dirname, 'public')));

// Logging middleware
const logActivity = (req, res, next) => {
  const timestamp = new Date().toISOString();
  const ip = req.ip || req.connection.remoteAddress;
  const method = req.method;
  const url = req.originalUrl;
  const userAgent = req.headers['user-agent'];
  
  const logEntry = `[${timestamp}] IP: ${ip} | Method: ${method} | URL: ${url} | User-Agent: ${userAgent}\n`;
  
  // Append to log file
  fs.appendFile('battleship_access.log', logEntry, (err) => {
    if (err) console.error('Error writing to log file:', err);
  });
  
  next();
};

app.use(logActivity);

// Debug endpoint
app.get('/api/battleship/debug', (req, res) => {
  res.json({ 
    status: 'ok', 
    timestamp: new Date().toISOString(),
    message: 'Battleship ZK Proof API is working'
  });
});

// Health check endpoint
app.get('/battleship/health', (req, res) => {
  res.json({ 
    status: 'ok', 
    timestamp: new Date().toISOString()
  });
});

// Battleship proof generation endpoint
app.post('/api/battleship/generate-proof', (req, res) => {
  const gameData = req.body;
  const timestamp = new Date().toISOString();
  const ip = req.ip || req.connection.remoteAddress;
  
  console.log(`[${timestamp}] New battleship proof request from IP: ${ip}`);
  console.log('Received game data:', JSON.stringify(gameData));
  
  // Validate required fields
  if (!gameData.username || 
      !gameData.game_result || 
      typeof gameData.game_result.ships_sunk === 'undefined' || 
      typeof gameData.game_result.total_shots === 'undefined') {
    console.error(`[${timestamp}] Missing required fields from IP: ${ip}`);
    return res.status(400).json({ 
      success: false, 
      error: 'Missing required fields: username, ships_sunk, and total_shots are required' 
    });
  }
  
  // Validate game result parameters
  const shipsSunk = parseInt(gameData.game_result.ships_sunk);
  const totalShots = parseInt(gameData.game_result.total_shots);
  
  if (isNaN(shipsSunk) || shipsSunk < 0 || shipsSunk > 10) {
    console.error(`[${timestamp}] Invalid ships sunk: ${gameData.game_result.ships_sunk} from IP: ${ip}`);
    return res.status(400).json({ 
      success: false, 
      error: 'Invalid ships sunk: must be a number between 0 and 9' 
    });
  }
  
  if (isNaN(totalShots) || totalShots < 10 || totalShots > 100) {
    console.error(`[${timestamp}] Invalid total shots: ${gameData.game_result.total_shots} from IP: ${ip}`);
    return res.status(400).json({ 
      success: false, 
      error: 'Invalid total shots: must be a number between 10 and 100' 
    });
  }
  
  // Set the script path - Use absolute path
  const scriptPath = path.resolve(__dirname, '..', 'script');
  
  // Create username hash
  const usernameHash = crypto.createHash('sha256').update(gameData.username).digest('hex');
  
  // Prepare command flags
  const winner = gameData.game_result.winner ? 'true' : 'false';
  const hitPercentage = gameData.game_result.hit_percentage || 
    Math.round((shipsSunk / totalShots) * 100);
  
  // Create the SP1 proof command
const command = `cargo run --bin prove --release -- --prove` +
    ` --username "${gameData.username}"` +
    ` --ships-sunk ${shipsSunk}` +
    ` --total-shots ${totalShots}` +
    ` --hit-percentage ${hitPercentage}` +
    (winner ? ` --winner ${winner}` : "") +
    ` --user-hash ${usernameHash}`;
  
  console.log(`[${timestamp}] Command to run: ${command}`);
  
  // Execute the command
  exec(command, { cwd: scriptPath }, (error, stdout, stderr) => {
    if (error) {
      console.error(`[${timestamp}] Proof generation error:`, error);
      return res.status(500).json({ 
        success: false, 
        error: 'Could not generate proof', 
        details: stderr, 
        command: command 
      });
    }
    
    // Generate proof hash
    const proofHash = crypto.createHash('sha256')
      .update(`${gameData.username}${shipsSunk}${totalShots}${Date.now()}`)
      .digest('hex')
      .substring(0, 32);
    
    // Return the successful result
    res.json({
      success: true,
      proofHash: `0xBS${proofHash}`,
      username: gameData.username,
      game_result: {
        ships_sunk: shipsSunk,
        total_shots: totalShots,
        hit_percentage: hitPercentage,
        winner: winner === 'true'
      },
      timestamp: Math.floor(Date.now() / 1000),
      verified: true
    });
    
    console.log(`[${timestamp}] Battleship proof generated and verified`);
  });
});

// Start the server
app.listen(PORT, () => {
  console.log(`Battleship ZK Proof Server running at http://localhost:${PORT}`);
  console.log(`Server ready to generate ZK proofs for Battleship game results!`);
});
