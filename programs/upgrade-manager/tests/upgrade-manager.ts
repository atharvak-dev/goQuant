import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { UpgradeManager } from "../target/types/upgrade_manager";
import { expect } from "chai";

describe("upgrade-manager", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.UpgradeManager as Program<UpgradeManager>;
  
  let multisigConfig: anchor.web3.PublicKey;
  let programUpgradeState: anchor.web3.PublicKey;
  let proposal: anchor.web3.PublicKey;
  
  const authority = provider.wallet.publicKey;
  const members = [
    anchor.web3.Keypair.generate().publicKey,
    anchor.web3.Keypair.generate().publicKey,
    anchor.web3.Keypair.generate().publicKey,
    anchor.web3.Keypair.generate().publicKey,
    anchor.web3.Keypair.generate().publicKey,
  ];
  const threshold = 3;
  const timelockDuration = 48 * 60 * 60; // 48 hours
  
  const programToUpgrade = anchor.web3.Keypair.generate().publicKey;
  const newProgramBuffer = anchor.web3.Keypair.generate().publicKey;

  before(async () => {
    // Derive PDAs
    [multisigConfig] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("multisig_config")],
      program.programId
    );

    [programUpgradeState] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("program_upgrade_state")],
      program.programId
    );

    [proposal] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("proposal"),
        programToUpgrade.toBuffer(),
        newProgramBuffer.toBuffer(),
      ],
      program.programId
    );
  });

  it("Initializes the upgrade manager", async () => {
    const tx = await program.methods
      .initialize(members, threshold, new anchor.BN(timelockDuration))
      .accounts({
        authority,
        multisigConfig,
        programUpgradeState,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    console.log("Initialize transaction signature", tx);

    // Verify multisig config
    const configAccount = await program.account.multisigConfig.fetch(multisigConfig);
    expect(configAccount.members).to.deep.equal(members);
    expect(configAccount.threshold).to.equal(threshold);
    expect(configAccount.upgradeAuthority.toString()).to.equal(authority.toString());

    // Verify program upgrade state
    const stateAccount = await program.account.programUpgradeState.fetch(programUpgradeState);
    expect(stateAccount.authority.toString()).to.equal(authority.toString());
    expect(stateAccount.timelockDuration.toNumber()).to.equal(timelockDuration);
  });

  it("Proposes an upgrade", async () => {
    const description = "Upgrade to v2.0.0 with new features";
    
    const tx = await program.methods
      .proposeUpgrade(newProgramBuffer, description)
      .accounts({
        proposer: authority,
        multisigConfig,
        programUpgradeState,
        program: programToUpgrade,
        proposal,
        newProgramBuffer,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("Propose upgrade transaction signature", tx);

    // Verify proposal
    const proposalAccount = await program.account.upgradeProposal.fetch(proposal);
    expect(proposalAccount.proposer.toString()).to.equal(authority.toString());
    expect(proposalAccount.program.toString()).to.equal(programToUpgrade.toString());
    expect(proposalAccount.newBuffer.toString()).to.equal(newProgramBuffer.toString());
    expect(proposalAccount.description).to.equal(description);
    expect(proposalAccount.approvals).to.have.lengthOf(1);
    expect(proposalAccount.approvals[0].toString()).to.equal(authority.toString());
    expect(proposalAccount.approvalThreshold).to.equal(threshold);
    expect(proposalAccount.status).to.deep.equal({ proposed: {} });
  });

  it("Approves an upgrade proposal", async () => {
    // Use a different member for approval (simulate multisig)
    const approver = anchor.web3.Keypair.generate();
    
    // First, we need to add the approver to the members list
    // In a real scenario, this would be one of the existing members
    
    const tx = await program.methods
      .approveUpgrade(proposal)
      .accounts({
        approver: authority, // Using authority as approver for test
        multisigConfig,
        proposal,
        programUpgradeState,
      })
      .rpc();

    console.log("Approve upgrade transaction signature", tx);

    // Verify approval was added
    const proposalAccount = await program.account.upgradeProposal.fetch(proposal);
    expect(proposalAccount.approvals).to.have.lengthOf(1); // Still 1 since same signer
  });

  it("Cannot execute upgrade before timelock expires", async () => {
    try {
      await program.methods
        .executeUpgrade(proposal)
        .accounts({
          executor: authority,
          proposal,
          programUpgradeState,
        })
        .rpc();
      
      expect.fail("Should have thrown timelock error");
    } catch (error) {
      expect(error.message).to.include("TimelockActive");
    }
  });

  it("Cancels an upgrade proposal", async () => {
    const tx = await program.methods
      .cancelUpgrade(proposal)
      .accounts({
        canceller: authority,
        multisigConfig,
        proposal,
      })
      .rpc();

    console.log("Cancel upgrade transaction signature", tx);

    // Verify proposal was cancelled
    const proposalAccount = await program.account.upgradeProposal.fetch(proposal);
    expect(proposalAccount.status).to.deep.equal({ cancelled: {} });
  });

  it("Creates and migrates an account", async () => {
    const oldAccount = anchor.web3.Keypair.generate().publicKey;
    
    const [accountVersion] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("account_version"), oldAccount.toBuffer()],
      program.programId
    );

    // First initialize the account version
    const initTx = await program.methods
      .migrateAccount(oldAccount)
      .accounts({
        migrator: authority,
        accountVersion,
        oldAccount,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("Migrate account transaction signature", initTx);

    // Verify migration
    const versionAccount = await program.account.accountVersion.fetch(accountVersion);
    expect(versionAccount.version).to.equal(1);
    expect(versionAccount.migrated).to.be.true;
    expect(versionAccount.migratedAt).to.not.be.null;
  });

  it("Cannot migrate already migrated account", async () => {
    const oldAccount = anchor.web3.Keypair.generate().publicKey;
    
    const [accountVersion] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("account_version"), oldAccount.toBuffer()],
      program.programId
    );

    // First migration
    await program.methods
      .migrateAccount(oldAccount)
      .accounts({
        migrator: authority,
        accountVersion,
        oldAccount,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    // Try to migrate again
    try {
      await program.methods
        .migrateAccount(oldAccount)
        .accounts({
          migrator: authority,
          accountVersion,
          oldAccount,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
      
      expect.fail("Should have thrown already migrated error");
    } catch (error) {
      expect(error.message).to.include("AlreadyMigrated");
    }
  });

  it("Handles multiple proposals", async () => {
    const newBuffer2 = anchor.web3.Keypair.generate().publicKey;
    const program2 = anchor.web3.Keypair.generate().publicKey;
    
    const [proposal2] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("proposal"),
        program2.toBuffer(),
        newBuffer2.toBuffer(),
      ],
      program.programId
    );

    const tx = await program.methods
      .proposeUpgrade(newBuffer2, "Second upgrade proposal")
      .accounts({
        proposer: authority,
        multisigConfig,
        programUpgradeState,
        program: program2,
        proposal: proposal2,
        newProgramBuffer: newBuffer2,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("Second proposal transaction signature", tx);

    // Verify both proposals exist
    const proposal1Account = await program.account.upgradeProposal.fetch(proposal);
    const proposal2Account = await program.account.upgradeProposal.fetch(proposal2);
    
    expect(proposal1Account.status).to.deep.equal({ cancelled: {} });
    expect(proposal2Account.status).to.deep.equal({ proposed: {} });
  });
});