import Koa, {ParameterizedContext, BaseContext } from 'koa';
//import Router from 'koa-router';
import Router from '@koa/router';
import {koaBody} from 'koa-body';
import logger from 'koa-logger';
import routers from './router';
import sleep from './lib/sleep';

const app = new Koa();
app.use(koaBody());
const router = new Router();
const port = 3000;


router.get('/sys/health', routers.health);
router.get('/', routers.ping);
router.get('/actions.json', routers.cors, routers.actionsJSON.getActionsJSON);

router.get('/createaccount', routers.cors, routers.createAccount.get);
router.options('/createaccount', routers.cors, routers.createAccount.get);
router.post('/createaccount', routers.cors, routers.createAccount.post);


app.use(logger());
app.use(router.routes());
app.use(router.allowedMethods());

app.listen(port, () => {
  console.log(`🚀 Server is running on port http://localhost:${port}/`);
});
