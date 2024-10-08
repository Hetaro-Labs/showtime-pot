import {ParameterizedContext} from 'koa';
import * as SA from '@solana/actions';
import sleep from '../lib/sleep';
import actionsJSON from './actionsjson';


function genMw(config: MyConf){
  return async function mw(ctx:ParameterizedContext, next:Function){
    ctx.state.config = config;
    await sleep(10);
    await next(ctx);
  }
}

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
  genMw,
  health,
  ping,
  actionsJSON,
}

