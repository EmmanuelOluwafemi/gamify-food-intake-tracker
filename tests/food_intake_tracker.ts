import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { FoodIntakeTracker } from "../target/types/food_intake_tracker";
import { expect } from "chai";

describe("food_intake_tracker", () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.FoodIntakeTracker as Program<FoodIntakeTracker>;
  const user = provider.wallet;

  // Derive PDA for food tracker account
  const [trackerPDA, trackerBump] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("food_tracker"), user.publicKey.toBuffer()],
    program.programId
  );

  describe("Initialize Tracker", () => {
    it("Successfully initializes a food tracker account", async () => {
      const tx = await program.methods
        .initialize()
        .accounts({
          user: user.publicKey,
        })
        .rpc();

      console.log("Initialize transaction signature:", tx);

      // Fetch the created account
      const trackerAccount = await program.account.foodTrackerAccount.fetch(trackerPDA);

      // Verify initial values
      expect(trackerAccount.owner.toString()).to.equal(user.publicKey.toString());
      expect(trackerAccount.currentStreak).to.equal(0);
      expect(trackerAccount.longestStreak).to.equal(0);
      expect(trackerAccount.totalPoints.toNumber()).to.equal(0);
      expect(trackerAccount.lastMealDate.toNumber()).to.equal(0);
      expect(trackerAccount.todayMeals.breakfast).to.be.false;
      expect(trackerAccount.todayMeals.lunch).to.be.false;
      expect(trackerAccount.todayMeals.dinner).to.be.false;
      expect(trackerAccount.bump).to.equal(trackerBump);

      console.log("✅ Tracker initialized successfully");
      console.log("   Owner:", trackerAccount.owner.toString());
      console.log("   Bump:", trackerAccount.bump);
      console.log("   PDA:", trackerPDA.toString());
    });

    it("Fails to initialize tracker twice for same user", async () => {
      try {
        await program.methods
          .initialize()
          .accounts({
            user: user.publicKey,
          })
          .rpc();

        expect.fail("Should have thrown an error");
      } catch (error) {
        expect(error).to.exist;
        console.log("✅ Correctly prevented duplicate initialization");
      }
    });
  });
});
