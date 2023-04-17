import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { CoodeStaking } from "../target/types/coode_staking";

describe("coode-staking", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.CoodeStaking as Program<CoodeStaking>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
