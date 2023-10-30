#![cfg(feature = "test-bpf")]

use anchor_lang::{solana_program::system_program, AccountDeserialize};

use {
    anchor_client::{
        anchor_lang::Discriminator,
        solana_sdk::{
            account::Account,
            commitment_config::CommitmentConfig,
            pubkey::Pubkey,
            signature::{Keypair, Signer},
            transaction::Transaction,
        },
        Client, Cluster,
    },
    solana_program_test::{tokio, ProgramTest},
    std::rc::Rc,
};

#[tokio::test]
async fn test_repro_zc_bug() {
    let mut pt = ProgramTest::new("zero_copy", zero_copy::id(), None);
    // pt.add_account(foo_pubkey, foo_account);
    let (mut banks_client, payer, recent_blockhash) = pt.start().await;

    let client = Client::new_with_options(
        Cluster::Debug,
        Rc::new(Keypair::new()),
        CommitmentConfig::processed(),
    );
    let zc_bug_pubkey = Pubkey::find_program_address(&[payer.pubkey().as_ref()], &zero_copy::ID).0;

    let program = client.program(zero_copy::id()).unwrap();
    let create_zc_bug_account_ix = program
        .request()
        .accounts(zero_copy::accounts::CreateZcBugAccount {
            zc_bug: zc_bug_pubkey,
            authority: payer.pubkey(),
            system_program: system_program::ID,
        })
        .args(zero_copy::instruction::CreateZcBugAccount)
        .instructions()
        .unwrap()
        .pop()
        .unwrap();

    let transaction = Transaction::new_signed_with_payer(
        &[create_zc_bug_account_ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    banks_client.process_transaction(transaction).await.unwrap();

    let zc_bug_account = banks_client
        .get_account(zc_bug_pubkey)
        .await
        .unwrap()
        .unwrap();
    let zc_bug = zero_copy::ZcBug::try_deserialize(&mut zc_bug_account.data.as_slice()).unwrap();
}
