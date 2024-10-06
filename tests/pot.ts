import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Pot } from "../target/types/pot";

const KEY_1:Uint8Array = new Uint8Array([95,65,216,25,216,150,128,112,33,157,106,229,77,58,90,253,44,118,126,18,69,169,189,132,19,83,135,229,89,220,12,120,4,156,172,  51,18,253,222,113,223,90,148,3,250,60,102,154,14,204,177,48,169,214,1,58,67,219,198,115,50,121,55,41]);
const KEY_2:Uint8Array = new Uint8Array([87,169,10,178,245,100,148,35,202,216,88,12,103,30,61,48,136,211,31,99,49,169,105,211,205,124,20,101,11,38,107,146,4,157,156,  182,82,137,54,187,90,80,199,201,123,94,29,55,70,156,186,24,84,85,59,100,239,105,154,225,158,78,170,54]);

const payer1 = anchor.web3.Keypair.fromSecretKey(KEY_1);
const payer2 = anchor.web3.Keypair.fromSecretKey(KEY_2);
const CONNECTION= new anchor.web3.Connection('http://127.0.0.1:8899');


function sleep(ms:number){
  return new Promise(resolve => setTimeout(resolve, ms));
}

describe("pot", () => {
  // Configure the client to use the local cluster.
  const provider1 = new anchor.AnchorProvider(CONNECTION, new anchor.Wallet(payer1), {commitment: "confirmed"});
  const provider2 = new anchor.AnchorProvider(CONNECTION, new anchor.Wallet(payer2), {commitment: "confirmed"});

  //anchor.setProvider(provider1);
  const program = anchor.workspace.Pot as Program<Pot>;

  it("Account Setup", async () => {
    console.log("Account Setup - Start!");
    await provider1.connection.requestAirdrop(payer1.publicKey, 100*anchor.web3.LAMPORTS_PER_SOL);
    await provider2.connection.requestAirdrop(payer2.publicKey, 100*anchor.web3.LAMPORTS_PER_SOL);
    await sleep(300);
    console.log("Account Setup - Done!");
  });

  it("Create Account 1", async () => {

    let [pdaProfile] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        anchor.utils.bytes.utf8.encode("profile"),
        payer1.publicKey.toBuffer()
      ],
      program.programId
    );

    let [pdaStakerList] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        anchor.utils.bytes.utf8.encode("staker_list"),
        payer1.publicKey.toBuffer()
      ],
      program.programId
    );


    try{
      const tx = await program.methods.createAccount({name: 'John Doe'})
        .accounts({signer: payer1.publicKey, profile: pdaProfile, staker_list: pdaStakerList })
        .signers([payer1])
        .rpc();
      console.log("Create Account TX 1:", tx);
    }catch(err){
      console.log(err);
    }

  });

  it("Create Account 2", async () => {

    let [pdaProfile] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        anchor.utils.bytes.utf8.encode("profile"),
        payer2.publicKey.toBuffer()
      ],
      program.programId
    );

    let [pdaStakerList] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        anchor.utils.bytes.utf8.encode("staker_list"),
        payer2.publicKey.toBuffer()
      ],
      program.programId
    );


    try{
      const tx = await program.methods.createAccount({name: 'Jane Doe'})
        .accounts({signer: payer2.publicKey, profile: pdaProfile, staker_list: pdaStakerList })
        .signers([payer2])
        .rpc();
      console.log("Create Account TX 2:", tx);
    }catch(err){
      console.log(err);
    }

  });



  it("End", async () => {
    await sleep(600 * 1000);
  });



});
