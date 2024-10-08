import {ParameterizedContext} from 'koa';
import sleep from '../lib/sleep';
import {MyConf} from '../type/myconf';



function genMw(config: MyConf){
  return async function mw(ctx:ParameterizedContext, next:Function){
    ctx.state.config = config;
    await sleep(10);
    await next(ctx);
  }
}

async function ping(ctx:ParameterizedContext, next:Function){
  ctx.body = 'Pong!\n' + ctx.config.text;
}

async function health(ctx:ParameterizedContext, next:Function){
  ctx.body = 'OK!\n';
}


export default {
  genMw,
  health,
  ping,
}

