//! Loads the REAL compiled `cu_envelope_demo.so` in-process via mollusk-svm
//! and measures actual compute_units_consumed at several values of n.
//! These are real numbers from real BPF execution, not estimates.

use mollusk_svm::Mollusk;
use solana_account::Account;
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;

const PROGRAM_ID: Pubkey = Pubkey::new_from_array([0x11u8; 32]);
const STATE_KEY: Pubkey = Pubkey::new_from_array([0x22u8; 32]);

fn build_ix_data(n: u32) -> Vec<u8> {
    let mut data = Vec::with_capacity(4 + (n as usize) * 8);
    data.extend_from_slice(&n.to_le_bytes());
    for i in 0..n {
        data.extend_from_slice(&(i as u64 + 1).to_le_bytes());
    }
    data
}

fn measure(mollusk: &Mollusk, n: u32) -> u64 {
    let data = build_ix_data(n);
    let ix = Instruction::new_with_bytes(PROGRAM_ID, &data, vec![AccountMeta::new(STATE_KEY, false)]);
    let accounts = vec![(STATE_KEY, Account::new(1_000_000, 12, &PROGRAM_ID))];
    let result = mollusk.process_instruction(&ix, &accounts);
    assert!(
        result.program_result.is_ok(),
        "n={n} failed: {:?}",
        result.program_result
    );
    result.compute_units_consumed
}

fn main() {
    if std::env::var("SBF_OUT_DIR").is_err() {
        std::env::set_var("SBF_OUT_DIR", "../program/target/deploy");
    }
    let mollusk = Mollusk::new(&PROGRAM_ID, "cu_envelope_demo");

    // Exhaustive, not sampled: every n from 1 to our operational cap, not a
    // few spot-checked points. A sparse sample can miss a kink; this can't.
    println!("n,cu");
    let mut prev: Option<(u32, u64)> = None;
    for n in 1u32..=49 {
        let cu = measure(&mollusk, n);
        println!("{n},{cu}");
        if let Some((prev_n, prev_cu)) = prev {
            let slope = (cu - prev_cu) as i64 / (n - prev_n) as i64;
            assert_eq!(slope, 7, "non-affine step detected at n={n}");
        }
        prev = Some((n, cu));
    }
    eprintln!("verified: exact affine slope=7 held across every consecutive step, n=1..=49");
}
