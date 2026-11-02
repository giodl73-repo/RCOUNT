import init,{evaluate,engine_info} from './pkg/rcount_package_workbench.js';
let ready=false;
self.onmessage=({data})=>{
  if(!ready||!Number.isSafeInteger(data?.id)||typeof data.request!=='string')return;
  try{self.postMessage({type:'result',id:data.id,report:evaluate(data.request)});}
  catch(error){self.postMessage({type:'failed',id:data.id,message:String(error)});}
};
try{await init();ready=true;self.postMessage({type:'ready',info:engine_info()});}
catch(error){self.postMessage({type:'failed',message:String(error)});}
