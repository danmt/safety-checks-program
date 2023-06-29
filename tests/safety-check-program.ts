import * as anchor from "@coral-xyz/anchor";
import {
  ComputeBudgetInstruction,
  ComputeBudgetProgram,
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import { assert } from "chai";
import { IDL, SafetyCheckManager } from "../target/types/safety_check_manager";

function getRandomArbitrary(min: number, max: number) {
  return Math.floor(Math.random() * (max - min) + min);
}

const TOKEN_PROGRAM_ID = new PublicKey(
  "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
);
const ASSOCIATED_TOKEN_PROGRAM_ID = new PublicKey(
  "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
);
const TOKEN_METADATA_PROGRAM_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);
const SAFETY_CHECK_MANAGER_PROGRAM_ID = new PublicKey(
  "4WJv7r8mzjydzhYRdG3yCGEmZmQT1KQUyxFrT1keaBWC"
);

describe("Test", () => {
  const provider = anchor.AnchorProvider.local();
  const safetyCheckProgram = new anchor.Program<SafetyCheckManager>(
    IDL,
    SAFETY_CHECK_MANAGER_PROGRAM_ID,
    provider
  );

  const siteId = getRandomArbitrary(0, 999999999999999).toString();
  const deviceId = getRandomArbitrary(0, 999999999999999).toString();
  const safetyCheckId = getRandomArbitrary(0, 999999999999999).toString();
  const safetyCheckName = "Safety Check";
  const safetyCheckSymbol = "SAFE";
  const safetyCheckUri = "https://www.google.com";
  const safetyCheckDurationInDays = 5;
  const bobKeypair = Keypair.generate();

  let sitePubkey: PublicKey;
  let inspectorPubkey: PublicKey;
  let devicePubkey: PublicKey;
  let safetyCheckPubkey: PublicKey;
  let safetyCheckMintPubkey: PublicKey;
  let deviceSafetyCheckVaultPubkey: PublicKey;
  let safetyCheckMetadataPubkey: PublicKey;
  let safetyCheckMasterEditionPubkey: PublicKey;

  before(async () => {
    [sitePubkey] = PublicKey.findProgramAddressSync(
      [Buffer.from("site", "utf-8"), Buffer.from(siteId, "utf-8")],
      safetyCheckProgram.programId
    );
    [inspectorPubkey] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("inspector", "utf-8"),
        Buffer.from(siteId, "utf-8"),
        provider.wallet.publicKey.toBuffer(),
      ],
      safetyCheckProgram.programId
    );
    [devicePubkey] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("device", "utf-8"),
        Buffer.from(siteId, "utf-8"),
        Buffer.from(deviceId, "utf-8"),
      ],
      safetyCheckProgram.programId
    );
    [safetyCheckPubkey] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("safety_check", "utf-8"),
        Buffer.from(siteId, "utf-8"),
        Buffer.from(deviceId, "utf-8"),
        Buffer.from(safetyCheckId, "utf-8"),
      ],
      safetyCheckProgram.programId
    );
    [safetyCheckMintPubkey] = PublicKey.findProgramAddressSync(
      [Buffer.from("safety_check_mint", "utf-8"), safetyCheckPubkey.toBuffer()],
      safetyCheckProgram.programId
    );
    [deviceSafetyCheckVaultPubkey] = PublicKey.findProgramAddressSync(
      [
        devicePubkey.toBuffer(),
        TOKEN_PROGRAM_ID.toBuffer(),
        safetyCheckMintPubkey.toBuffer(),
      ],
      ASSOCIATED_TOKEN_PROGRAM_ID
    );
    [safetyCheckMetadataPubkey] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata", "utf-8"),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        safetyCheckMintPubkey.toBuffer(),
      ],
      TOKEN_METADATA_PROGRAM_ID
    );
    [safetyCheckMasterEditionPubkey] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata", "utf-8"),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        safetyCheckMintPubkey.toBuffer(),
        Buffer.from("edition", "utf-8"),
      ],
      TOKEN_METADATA_PROGRAM_ID
    );
  });

  it("should create a site", async () => {
    // act
    await safetyCheckProgram.methods
      .createSite(siteId)
      .accounts({
        authority: provider.wallet.publicKey,
        site: sitePubkey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    // assert
    const siteAccount = await safetyCheckProgram.account.site.fetchNullable(
      sitePubkey
    );
    assert.notEqual(siteAccount, null);
    assert.equal(siteAccount.authority.equals(provider.wallet.publicKey), true);
    assert.equal(siteAccount.siteId, siteId);
  });

  it("should create a inspector", async () => {
    // act
    await safetyCheckProgram.methods
      .createInspector(siteId)
      .accounts({
        authority: provider.wallet.publicKey,
        site: sitePubkey,
        inspector: inspectorPubkey,
        systemProgram: SystemProgram.programId,
        owner: bobKeypair.publicKey,
      })
      .rpc({ commitment: "confirmed" });
    // assert
    const inspectorAccount =
      await safetyCheckProgram.account.inspector.fetchNullable(inspectorPubkey);
    assert.notEqual(inspectorAccount, null);
    assert.equal(inspectorAccount.owner.equals(bobKeypair.publicKey), true);
    assert.equal(inspectorAccount.siteId, siteId);
  });

  it("should create a device", async () => {
    // act
    await safetyCheckProgram.methods
      .createDevice(siteId, deviceId)
      .accounts({
        authority: provider.wallet.publicKey,
        site: sitePubkey,
        device: devicePubkey,
        systemProgram: SystemProgram.programId,
      })
      .rpc({ commitment: "confirmed" });
    // assert
    const deviceAccount = await safetyCheckProgram.account.device.fetchNullable(
      devicePubkey
    );
    assert.notEqual(deviceAccount, null);
    assert.equal(deviceAccount.siteId, siteId);
    assert.equal(deviceAccount.deviceId, deviceId);
    assert.equal(deviceAccount.lastSafetyCheck, null);
    assert.equal(deviceAccount.expiresAt, null);
  });

  it("should create a safety check", async () => {
    // act
    await safetyCheckProgram.methods
      .createSafetyCheck(
        siteId,
        deviceId,
        safetyCheckId,
        safetyCheckName,
        safetyCheckSymbol,
        safetyCheckUri,
        new anchor.BN(safetyCheckDurationInDays)
      )
      .accounts({
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        metadataProgram: TOKEN_METADATA_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
        authority: provider.wallet.publicKey,
        site: sitePubkey,
        device: devicePubkey,
        inspector: inspectorPubkey,
        safetyCheck: safetyCheckPubkey,
        safetyCheckMint: safetyCheckMintPubkey,
        deviceSafetyCheckVault: deviceSafetyCheckVaultPubkey,
        safetyCheckMetadata: safetyCheckMetadataPubkey,
        safetyCheckMasterEdition: safetyCheckMasterEditionPubkey,
      })
      .preInstructions([
        ComputeBudgetProgram.setComputeUnitLimit({ units: 300_000 }),
      ])
      .rpc({ commitment: "confirmed" });
    // assert
    const deviceAccount = await safetyCheckProgram.account.device.fetchNullable(
      devicePubkey
    );
    const safetyCheckAccount =
      await safetyCheckProgram.account.safetyCheck.fetchNullable(
        safetyCheckPubkey
      );
    assert.notEqual(safetyCheckAccount, null);
    assert.equal(safetyCheckAccount.siteId, siteId);
    assert.equal(safetyCheckAccount.deviceId, deviceId);
    assert.equal(safetyCheckAccount.safetyCheckId, safetyCheckId);
    assert.equal(safetyCheckAccount.inspector.equals(inspectorPubkey), true);
    assert.equal(
      safetyCheckAccount.durationInDays.eq(
        new anchor.BN(safetyCheckDurationInDays)
      ),
      true
    );
    assert.equal(
      safetyCheckAccount.createdAt
        .add(new anchor.BN(safetyCheckDurationInDays).mul(new anchor.BN(86400)))
        .eq(safetyCheckAccount.expiresAt),
      true
    );
    assert.equal(
      deviceAccount.expiresAt.eq(safetyCheckAccount.expiresAt),
      true
    );
    assert.equal(deviceAccount.lastSafetyCheck.equals(safetyCheckPubkey), true);
  });
});
