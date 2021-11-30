import * as anchor from "@project-serum/anchor";
import {
    PublicKey,
    TransactionInstruction,
    Signer, 
    Transaction,
    SYSVAR_RENT_PUBKEY,
    Connection,
    ConfirmOptions,
    TransactionSignature,
    sendAndConfirmRawTransaction,
    Keypair
} from "@solana/web3.js";
import { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID, } from "@solana/spl-token";
import { TokenInstructions } from "@project-serum/serum";
const serumCmn = require("@project-serum/common");

export const SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID = new PublicKey(
    "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
);



export async function createMint(
    provider: anchor.Provider,
    authority: PublicKey,
    decimals: number = 6
): Promise<PublicKey> {
    const mint = anchor.web3.Keypair.generate();
    const instructions = await createMintInstructions(
        provider,
        authority,
        mint.publicKey,
        decimals
    );

    const tx = new anchor.web3.Transaction();
    tx.add(...instructions);

    await provider.send(tx, [mint]);

    return mint.publicKey;
}


async function createMintInstructions(
    provider: anchor.Provider,
    authority: PublicKey,
    mint: PublicKey,
    decimals: number
): Promise<TransactionInstruction[]> {
    let instructions = [
        anchor.web3.SystemProgram.createAccount({
            fromPubkey: provider.wallet.publicKey,
            newAccountPubkey: mint,
            space: 82,
            lamports: await provider.connection.getMinimumBalanceForRentExemption(82),
            programId: TOKEN_PROGRAM_ID,
        }),
        TokenInstructions.initializeMint({
            mint,
            decimals,
            mintAuthority: authority,
        }),
    ];
    return instructions;
}


export async function createTokenAccount(
    provider: anchor.Provider,
    mint: PublicKey,
    owner: PublicKey
): Promise<PublicKey> {
    const vault = anchor.web3.Keypair.generate();
    const tx = new anchor.web3.Transaction();
    tx.add(
        ...(await createTokenAccountInstrs(provider, vault.publicKey, mint, owner))
    );
    await provider.send(tx, [vault]);
    return vault.publicKey;
}

async function createTokenAccountInstrs(
    provider: anchor.Provider,
    newAccountPubkey: PublicKey,
    mint: PublicKey,
    owner: PublicKey,
    lamports?: number
) {
    if (lamports === undefined) {
        lamports = await provider.connection.getMinimumBalanceForRentExemption(165);
    }
    return [
        anchor.web3.SystemProgram.createAccount({
            fromPubkey: provider.wallet.publicKey,
            newAccountPubkey,
            space: 165,
            lamports,
            programId: TOKEN_PROGRAM_ID,
        }),
        TokenInstructions.initializeAccount({
            account: newAccountPubkey,
            mint,
            owner,
        }),
    ];
}

export async function setUpAta(
    provider: anchor.Provider,
    user_payer: Signer,
    mint: PublicKey
): Promise<PublicKey> {
    const [ix, address] = await createAssociatedTokenAccountIx(
        user_payer.publicKey,
        user_payer.publicKey,
        mint
    );

    const tx = new Transaction();
    tx.add(ix);

    if (
        (await checkIfAccountExists(provider.connection, address)) == false
    ) {
        await send(provider, tx, user_payer.publicKey, [user_payer]);
    }

    return address;
}


export async function createAssociatedTokenAccountIx(
    fundingAddress: PublicKey,
    walletAddress: PublicKey,
    splTokenMintAddress: PublicKey
): Promise<[TransactionInstruction, PublicKey]> {
    const associatedTokenAddress = await findAssociatedTokenAddress(
        walletAddress,
        splTokenMintAddress
    );
    const systemProgramId = new PublicKey("11111111111111111111111111111111");
    const keys = [
        {
            pubkey: fundingAddress,
            isSigner: true,
            isWritable: true,
        },
        {
            pubkey: associatedTokenAddress,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: walletAddress,
            isSigner: false,
            isWritable: false,
        },
        {
            pubkey: splTokenMintAddress,
            isSigner: false,
            isWritable: false,
        },
        {
            pubkey: systemProgramId,
            isSigner: false,
            isWritable: false,
        },
        {
            pubkey: TokenInstructions.TOKEN_PROGRAM_ID,
            isSigner: false,
            isWritable: false,
        },
        {
            pubkey: SYSVAR_RENT_PUBKEY,
            isSigner: false,
            isWritable: false,
        },
    ];
    const ix = new TransactionInstruction({
        keys,
        programId: ASSOCIATED_TOKEN_PROGRAM_ID,
        data: Buffer.from([]),
    });
    return [ix, associatedTokenAddress];
}

export async function findAssociatedTokenAddress(
    owner: PublicKey,
    tokenMintAddress: PublicKey
): Promise<PublicKey> {
    let res = (await PublicKey.findProgramAddress(
        [
            owner.toBuffer(),
            TOKEN_PROGRAM_ID.toBuffer(),
            tokenMintAddress.toBuffer(),
        ],
        SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID
    ))[0];

    return res;
}

export async function checkIfAccountExists(
    connection: Connection,
    account: PublicKey
): Promise<boolean> {
    const acc = await connection.getAccountInfo(account);
    return acc != null;
}


export async function send(
    provider: anchor.Provider,
    tx: Transaction,
    payer: PublicKey,
    signers?: Array<Signer | undefined>,
    opts?: ConfirmOptions
): Promise<TransactionSignature> {
    if (signers === undefined) {
        signers = [];
    }

    let { blockhash } = await provider.connection.getRecentBlockhash();
    tx.feePayer = payer;
    tx.recentBlockhash = blockhash;

    // await provider.wallet.signTransaction(tx);
    signers.forEach((kp: Signer | undefined) => {
        if (kp !== undefined) {
            tx.partialSign(kp);
        }
    });

    const rawTx = tx.serialize();

    return await sendAndConfirmRawTransaction(
        provider.connection,
        rawTx,
        opts
    );
}


export async function mintTo(
    provider: anchor.Provider,
    mint: PublicKey,
    to: PublicKey,
    amount: number,
): Promise<void> {
    const instruction = TokenInstructions.mintTo({
        mint,
        destination: to,
        amount,
        mintAuthority: provider.wallet.publicKey,
    });

    const tx = new anchor.web3.Transaction();
    tx.add(instruction);

    let sig = await provider.send(tx);
}

export const FACTOR = 1_000_000.0;
export function decimalToU64(n: number): number {
    let n1 = n * FACTOR;
    let n2 = Math.trunc(n1);
    return n2;
}


export async function getTokenAccount(provider:any, addr:any) {
    return await serumCmn.getTokenAccount(provider, addr);
}