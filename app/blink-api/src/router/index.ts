import {ParameterizedContext} from 'koa';
import * as SA from '@solana/actions';
import sleep from '../lib/sleep';
import demo from './demo';
import actionsJSON from './actionsjson';
import createAccount from './createaccount';
import createEvent from './createevent';
import joinEvent from './joinevent';



async function ping(ctx:ParameterizedContext, next:Function){
  ctx.body = 'Pong!\n';
}

async function health(ctx:ParameterizedContext, next:Function){
  ctx.body = 'OK!\n';
}

// solana actions CORS headers
async function cors(ctx:ParameterizedContext, next:Function){
  ctx.response.set(SA.ACTIONS_CORS_HEADERS);
  await next(ctx);
  
}




export default {
  cors,
  health,
  ping,
  demo,
  actionsJSON,
  createAccount,
  createEvent,
  joinEvent,
}

