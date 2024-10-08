import Koa, {ParameterizedContext, BaseContext } from 'koa';
import Router from 'koa-router';
import {koaBody} from 'koa-body';
import logger from 'koa-logger';
import routers from './router';
import sleep from './lib/sleep';

const app = new Koa();
const router = new Router();
const port = 3000;


router.get('/sys/health', routers.health);
router.get('/', routers.ping);

app.use(logger());
app.use(router.routes());

app.listen(port, () => {
  console.log(`🚀 Server is running on port http://localhost:${port}/`);
});
