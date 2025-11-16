// Patent Pending Copyright © 2025 Xeris Web Co. All rights reserved.
// XerisCoin Local Wallet - Connects to 127.0.0.1:4001 Only

const { Connection, Keypair, Transaction, SystemProgram, PublicKey } = require('@solana/web3.js');
const fs = require('fs');
const { Program, AnchorProvider, web3 } = require('@project-serum/anchor');
const idl = require('./xeris_stake_idl.json');  // Assume bundled

async function getConnection() {
    // Local Alpha: Hardcode local endpoint
    const conn = new Connection('http://127.0.0.1:4001', 'confirmed');
    // Mock version check for local
    console.log('Connected to Local Alpha Node');
    return conn;
}

async function sendXRS(fromKeypairPath, toPubkeyStr, amount) {
    const connection = await getConnection();
    const from = Keypair.fromSecretKey(Uint8Array.from(JSON.parse(fs.readFileSync(fromKeypairPath, 'utf8'))));
    const balance = await connection.getBalance(from.publicKey);  // Local balance
    const lamports = amount * 1e9;
    const fee = lamports * 0.001; // 0.1% fee
    if (balance < lamports + fee) {
        throw new Error(`Insufficient local balance: ${balance / 1e9} XRS`);
    }
    const to = new PublicKey(toPubkeyStr);
    const tx = new Transaction().add(
        SystemProgram.transfer({
            fromPubkey: from.publicKey,
            toPubkey: to,
            lamports
        })
    );
    const signature = await connection.sendTransaction(tx, [from]);
    await connection.confirmTransaction(signature);
    console.log(`Local Send: ${amount} XRS to ${toPubkeyStr}. Fee: ${fee / 1e9} XRS. Sig: ${signature}`);
}

async function stakeXRS(fromKeypairPath, amount) {
    const connection = await getConnection();
    const from = Keypair.fromSecretKey(Uint8Array.from(JSON.parse(fs.readFileSync(fromKeypairPath, 'utf8'))));
    const provider = new AnchorProvider(connection, { publicKey: from.publicKey, signTransaction: async (tx) => tx.sign(from) }, {});
    const program = new Program(idl, 'XerisStakeProgram', provider);
    const [stakeAccount] = await web3.PublicKey.findProgramAddress(
        [Buffer.from('stake'), from.publicKey.toBuffer()],
        program.programId
    );
    await program.rpc.initializeStake(new anchor.BN(amount * 1e9), {
        accounts: {
            stakeAccount,
            owner: from.publicKey,
            systemProgram: web3.SystemProgram.programId
        },
        signers: [from]
    });
    console.log(`Local Stake: ${amount} XRS (Patent Pending)`);
}

// ... (unstake similar, local conn)

async function claimAirdrop(address) {
    const response = await fetch(`http://127.0.0.1:4001/airdrop/${address}/1000000000000`, { method: 'POST' });
    const result = await response.json();
    if (result.error) {
        throw new Error(`Local Airdrop failed: ${result.error}`);
    }
    console.log('Local Airdrop Claimed: 1,000 XRS');
}

if (require.main === module) {
    const [, , command, ...args] = process.argv;
    if (command === 'send') {
        const [, fromKeypairPath, toPubkeyStr, amount] = args;
        sendXRS(fromKeypairPath, toPubkeyStr, parseFloat(amount)).catch(console.error);
    } else if (command === 'stake') {
        const [, fromKeypairPath, amount] = args;
        stakeXRS(fromKeypairPath, parseFloat(amount)).catch(console.error);
    } else if (command === 'unstake') {
        // similar
    } else if (command === 'airdrop') {
        const [, address] = args;
        claimAirdrop(address).catch(console.error);
    } else {
        console.log('Local Alpha Usage: node wallet.js <command> <args>');
        console.log('Commands: send <keypath> <to> <amt> | stake <keypath> <amt> | airdrop <addr>');
    }
}