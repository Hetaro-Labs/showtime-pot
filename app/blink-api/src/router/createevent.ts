import {ParameterizedContext} from 'koa';
import * as SA from '@solana/actions';
import sleep from '../lib/sleep';
import * as W3 from '@solana/web3.js';


// Testing site:
// https://dial.to/devnet?action=solana-action:http://127.0.0.1:3000/createaccount



async function get (ctx:ParameterizedContext, next:Function){
  const payload: SA.ActionGetResponse = {
    type: "action",
    title: "PoT - Create Event",
    icon: 'https://cdn.hetaro.com/solana-actions/showtime-watch.png',
    description: "Create an evnet at your PoT network",
    label: "Create Event",
    links: {
      actions: [
        {
          type: 'transaction',
          label: "Create Event",
          href: '/createevent',
          parameters: [
            {
              type: 'text',
              name: "event_name",
              label: "Event name", 
              required: true,
              patternDescription: 'Your event name',
            },
          ]

        },
       ]
    }
  };
  ctx.body = payload;
}

async function post (ctx:ParameterizedContext, next:Function){

  let payerPubkey:W3.PublicKey;
  try{
    payerPubkey = new W3.PublicKey(ctx.request.body?.account);
  }catch(err){
    console.log(err);
    let payload:SA.ActionError = {
      message: 'Wallet Address Error',
    } 
    ctx.body = payload;
    return;
  }

  let tx = new W3.Transaction();
  let tiMemo = new W3.TransactionInstruction({    
    programId: new W3.PublicKey(SA.MEMO_PROGRAM_ID),    
    data: Buffer.from('Create Event', 'utf8'),    
    keys: [],    
  }); 
  tx.add(tiMemo);
  tx.feePayer = payerPubkey;
  const conn = new W3.Connection('https://api.devnet.solana.com');

  const latestBlockhash = await conn.getLatestBlockhash();
  tx.recentBlockhash = latestBlockhash.blockhash;

  try{
    const payload: SA.ActionPostResponse = await SA.createPostResponse({
      fields: {
        transaction: tx,
        message: "Transaction created",
      },
    });

    ctx.body = payload;
  }catch(err){
    console.log(err);

    let payload:SA.ActionError = {
      message: 'Error',
    } 
    ctx.body = payload;
    return;

  }
}

export default {
  get,
  post,
}

