import Orbiter from '../packages/renderer/src/plugins/orbiter/orbiter.ts';

console.log('ici');
(new Orbiter()).setUpSite().then(x=>console.log(x));