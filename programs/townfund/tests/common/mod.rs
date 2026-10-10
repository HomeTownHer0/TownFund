//! Test bench: a fresh in-memory Solana chain (LiteSVM) with TownFund
//! loaded and a mock USDC mint at the exact address the program expects.
#![allow(dead_code)]

use {
    anchor_lang::{
        prelude::Pubkey, solana_program::system_program, AccountDeserialize, InstructionData,
        ToAccountMetas,
    },
    litesvm::LiteSVM,
    solana_account::Account,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
    townfund::{ProjectKind, PROJECT_SEED, USDC_MINT, VAULT_SEED},
};

pub const TOKEN_PROGRAM: Pubkey = anchor_spl::token::ID;
pub const DAY: i64 = 86_400;
pub const USDC: u64 = 1_000_000;

pub struct Bench {
    pub svm: LiteSVM,
    /// Holds the mock USDC mint authority (can mint test USDC).
    pub mint_authority: Keypair,
}

impl Bench {
    pub fn new() -> Self {
        let mut svm = LiteSVM::new();
        let bytes = include_bytes!(concat!(
            env!("CARGO_TARGET_TMPDIR"),
            "/../deploy/townfund.so"
        ));
        svm.add_program(townfund::id(), bytes).unwrap();
        let mint_authority = Keypair::new();
        let mut bench = Self { svm, mint_authority };
        let authority = bench.mint_authority.pubkey();
        bench.put_mint(USDC_MINT, authority, 6);
        bench
    }

    /// A funded wallet.
    pub fn wallet(&mut self) -> Keypair {
        let k = Keypair::new();
        self.svm.airdrop(&k.pubkey(), 10_000_000_000).unwrap();
        k
    }

    /// Write an SPL Token mint account directly at `address` (82 bytes).
    pub fn put_mint(&mut self, address: Pubkey, authority: Pubkey, decimals: u8) {
        let mut data = vec![0u8; 82];
        data[0..4].copy_from_slice(&1u32.to_le_bytes()); // mint_authority: Some
        data[4..36].copy_from_slice(authority.as_ref());
        // supply = 0 (bytes 36..44)
        data[44] = decimals;
        data[45] = 1; // is_initialized
        // freeze_authority: None (bytes 46..82 all zero)
        let lamports = self.svm.minimum_balance_for_rent_exemption(82);
        self.svm
            .set_account(
                address,
                Account { lamports, data, owner: TOKEN_PROGRAM, executable: false, rent_epoch: 0 },
            )
            .unwrap();
    }

    pub fn now(&self) -> i64 {
        self.svm.get_sysvar::<anchor_lang::prelude::Clock>().unix_timestamp
    }

    /// Send one instruction. Ok, or the custom fault code if refused.
    pub fn send(
        &mut self,
        ix: anchor_lang::solana_program::instruction::Instruction,
        signers: &[&Keypair],
    ) -> Result<(), Fault> {
        let payer = signers[0].pubkey();
        let blockhash = self.svm.latest_blockhash();
        let msg = Message::new_with_blockhash(&[ix], Some(&payer), &blockhash);
        let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
        let res = self.svm.send_transaction(tx);
        self.svm.expire_blockhash();
        res.map(|_| ()).map_err(|e| Fault::from_err(&e.err, e.meta.logs))
    }

    pub fn project(&self, address: Pubkey) -> townfund::Project {
        let acct = self.svm.get_account(&address).expect("project missing");
        let mut data: &[u8] = &acct.data;
        townfund::Project::try_deserialize(&mut data).unwrap()
    }

    /// (mint, owner, amount) of an SPL token account.
    pub fn token_account(&self, address: Pubkey) -> (Pubkey, Pubkey, u64) {
        let acct = self.svm.get_account(&address).expect("token account missing");
        assert_eq!(acct.owner, TOKEN_PROGRAM);
        let d = &acct.data;
        (
            Pubkey::try_from(&d[0..32]).unwrap(),
            Pubkey::try_from(&d[32..64]).unwrap(),
            u64::from_le_bytes(d[64..72].try_into().unwrap()),
        )
    }
}

/// Why a transaction was refused.
#[derive(Debug)]
pub struct Fault {
    /// Custom code (TownFund faults are 6000+), if any.
    pub code: Option<u32>,
    pub logs: Vec<String>,
}

impl Fault {
    fn from_err(err: &solana_transaction_error::TransactionError, logs: Vec<String>) -> Self {
        use solana_instruction_error::InstructionError;
        use solana_transaction_error::TransactionError;
        let code = match err {
            TransactionError::InstructionError(_, InstructionError::Custom(c)) => Some(*c),
            _ => None,
        };
        Self { code, logs }
    }
}

/// On-chain code for a TownFund fault.
pub fn code(e: townfund::error::TownFundError) -> u32 {
    anchor_lang::error::ERROR_CODE_OFFSET + e as u32
}

pub fn project_address(creator: &Pubkey, project_id: u64) -> Pubkey {
    Pubkey::find_program_address(
        &[PROJECT_SEED, creator.as_ref(), &project_id.to_le_bytes()],
        &townfund::id(),
    )
    .0
}

pub fn vault_address(project: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[VAULT_SEED, project.as_ref()], &townfund::id()).0
}

/// Inputs for initialize_project, with sensible defaults.
#[derive(Clone)]
pub struct NewProject {
    pub project_id: u64,
    pub kind: ProjectKind,
    pub target: u64,
    pub duration_secs: i64,
    pub title: String,
    pub candidates: Vec<Pubkey>,
    pub mint: Pubkey,
}

impl Default for NewProject {
    fn default() -> Self {
        Self {
            project_id: 1,
            kind: ProjectKind::Creator,
            target: 100 * USDC,
            duration_secs: 7 * DAY,
            title: "New roof for the Salina food pantry".into(),
            candidates: vec![],
            mint: USDC_MINT,
        }
    }
}

pub fn initialize_project_ix(
    creator: &Pubkey,
    p: &NewProject,
) -> anchor_lang::solana_program::instruction::Instruction {
    let project = project_address(creator, p.project_id);
    anchor_lang::solana_program::instruction::Instruction::new_with_bytes(
        townfund::id(),
        &townfund::instruction::InitializeProject {
            project_id: p.project_id,
            kind: p.kind,
            target: p.target,
            duration_secs: p.duration_secs,
            title: p.title.clone(),
            candidates: p.candidates.clone(),
        }
        .data(),
        townfund::accounts::InitializeProject {
            creator: *creator,
            project,
            mint: p.mint,
            vault: vault_address(&project),
            token_program: TOKEN_PROGRAM,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    )
}
