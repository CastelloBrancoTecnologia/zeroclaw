const {chromium}=require('playwright');
const fs=require('node:fs'),path=require('node:path');
const out=__dirname;
const prefix='channels.xmpp.private';
const values={username:'bot@example.org',server:'xmpp.example.org',port:'5223',protocol:'direct_tls',resource:'zeroclaw',max_message_chars:'16000',connect_timeout_secs:'30',python_path:'/opt/zeroclaw-xmpp/bin/python',omemo_enabled:'false',omemo_trusted_devices:'[]',enabled:'false'};
const writes=[]; let browser,page;let peers='[]';let policyCreated=false;let policyChannel='';
function entry(p,kind,value,tab){return {path:p,kind,value,type_hint:kind,is_secret:false,populated:true,category:'channels',section:'channels',tab,is_env_overridden:false,enum_variants:[],multiline:false};}
function entries(){return [
 ...Object.entries(values).map(([k,v])=>({...entry(prefix+'.'+k,['enabled','omemo_enabled'].includes(k)?'bool':['port','max_message_chars','connect_timeout_secs'].includes(k)?'integer':k==='protocol'?'enum':k==='omemo_trusted_devices'?'string-array':'string',v,['enabled','username','server','port','protocol'].includes(k)?'Connection':k==='enabled'?'Behavior':'Advanced'),...(k==='protocol'?{enum_variants:['direct_tls','starttls']}:{})})),
 {...entry(prefix+'.password','string',undefined,'Connection'),is_secret:true},
 ...(policyCreated?[entry('peer_groups.xmpp_private.channel','string',policyChannel),entry('peer_groups.xmpp_private.external_peers','string-array',peers),entry('peer_groups.xmpp_private.ignore','string-array','["mallory@example.org"]'),entry('peer_groups.xmpp_private.agents','string-array','[]')]:[]),entry('peer_groups.other.channel','string','irc.other')];}
const sections=[{key:'channels',label:'Channels',help:'Messaging channels',has_picker:true,completed:true,ready:true,group:'Foundation',is_quickstart:false,shape:'typed_family_map'}, {key:'peer_groups',label:'Peer groups',help:'Authorization',has_picker:true,completed:true,ready:true,group:'Foundation',is_quickstart:false,shape:'one_tier_alias_map'}];
(async()=>{
 browser=await chromium.launch({channel:'chrome',headless:false});
 const ctx=await browser.newContext({viewport:{width:1440,height:1000}});
 await ctx.addInitScript(()=>localStorage.setItem('zeroclaw_token','local-ui-fixture'));
 await ctx.route('**/api/**',async route=>{
  const req=route.request(),u=new URL(req.url()),p=u.pathname;let data={};
  if(p==='/api/config/list') {const pre=u.searchParams.get('prefix');data={entries:entries().filter(e=>!pre||e.path.startsWith(pre+'.')),drifted:[]};}
  else if(p==='/api/config/sections')data={sections};
  else if(p==='/api/config/sections/channels')data={section:'channels',items:[{key:'xmpp',label:'XMPP',badge:'configured'}]};
  else if(p==='/api/config/map-keys')data={keys:u.searchParams.get('path')==='peer_groups'?(policyCreated?['xmpp_private','other']:['other']):['private']};
  else if(p==='/api/config/map-key'&&req.method()==='POST'){if(u.searchParams.get('key')!=='xmpp_private')throw Error('wrong policy key');policyCreated=true;data={path:'peer_groups',key:'xmpp_private',created:true};}
  else if(p==='/api/config/schema')data={type:'object',properties:{}};
  else if(p==='/api/config/drift')data={drifted:[]};
  else if(p==='/api/config/reload-status')data={pending_reload:false};
  else if(p==='/api/config/catalog')data={channels:[{name:'xmpp',label:'XMPP'}],model_providers:[]};
  else if(p==='/api/config'&&req.method()==='PATCH') {const ops=req.postDataJSON();for(const op of ops){if(op.path==='peer_groups.xmpp_private.channel')policyChannel=op.value;if(op.path==='peer_groups.xmpp_private.external_peers')peers=JSON.stringify(op.value);if(op.path.startsWith(prefix+'.')&&!op.path.endsWith('.password'))values[op.path.slice(prefix.length+1)]=String(op.value);}writes.push(ops);data={saved:true,results:ops,warnings:[]};}
  else if(p==='/api/status')data={version:'0.8.6',locale:'en',model:'fixture',uptime_seconds:60,components:[],health:{},cost:{}};
  else if(p==='/api/agents')data=[];
  else if(p.includes('update')||p.includes('upgrade'))data={available:false,current_version:'0.8.6'};
  await route.fulfill({status:200,contentType:'application/json',body:JSON.stringify(data)});
 });
 page=await ctx.newPage();const errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.goto((process.env.XMPP_UI_URL || 'http://127.0.0.1:5178')+'/config/channels/xmpp/private');
 await page.locator('[id="'+prefix+'.username"]').waitFor({timeout:90000});
 fs.mkdirSync(out,{recursive:true});
 await page.screenshot({path:path.join(out,'xmpp-connection-desktop.png')});
 const protocol=page.locator('[id="'+prefix+'.protocol"]');
 await protocol.click();await page.getByRole('option',{name:'starttls',exact:true}).click();
 const save=async()=>{await Promise.all([page.waitForResponse(r=>r.url().endsWith('/api/config')&&r.request().method()==='PATCH'),page.getByRole('button',{name:'Save',exact:true}).click()]);await page.getByText(/Saved .*field/).waitFor();};
 await save();await page.reload();await protocol.waitFor();
 if((await protocol.innerText()).trim()!=='starttls'||values.port!=='5223')throw Error('transport persistence or independent port failed');
 if(await page.locator('input[id="'+prefix+'.password"]').count())throw Error('saved password exposed as input');
 await page.getByRole('button',{name:'Change',exact:true}).click();
 const password=page.locator('[id="'+prefix+'.password"]');
 if(await password.inputValue()!==''||await password.getAttribute('type')!=='password')throw Error('secret editor is not blank/masked');
 await password.fill('replacement-fixture');await save();await page.reload();await protocol.waitFor();if(await page.locator('input[id="'+prefix+'.password"]').count())throw Error('saved replacement was returned');
 await page.getByRole('tab',{name:'Advanced',exact:true}).click();
 const omemo=page.locator('[id="'+prefix+'.omemo_enabled"]');await omemo.waitFor();await omemo.click();await save();
 if(values.omemo_enabled!=='true')throw Error('OMEMO option did not persist');await page.screenshot({path:path.join(out,'xmpp-omemo-desktop.png')});
 await Promise.all([page.waitForResponse(r=>r.url().endsWith('/api/config')&&r.request().method()==='PATCH'),page.getByRole('button',{name:'Enable',exact:true}).click()]);if(values.enabled!=='true')throw Error('enabled not saved');
 await page.getByRole('tab',{name:'Allowed users',exact:true}).click();
 await page.getByRole('button',{name:'Create authorization policy for this channel',exact:true}).click();await page.getByText('External Peers',{exact:true}).waitFor();if(policyChannel!=='xmpp.private')throw Error('policy not scoped to alias');
 if(await page.locator('[id="peer_groups.other.channel"]').count())throw Error('unrelated peer group exposed');
 await page.getByRole('button',{name:'Add',exact:true}).first().click();await page.locator('[id="peer_groups.xmpp_private.external_peers"] input').first().fill('user@example.org');await save();if(peers!=='["user@example.org"]')throw Error('canonical peers not saved');await page.screenshot({path:path.join(out,'xmpp-allowed-users-desktop.png')});
 await page.setViewportSize({width:390,height:844});
 await page.screenshot({path:path.join(out,'xmpp-allowed-users-mobile.png')});
 const overflow=await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth);
 if(overflow||errors.length)throw Error(JSON.stringify({overflow,errors}));
 fs.writeFileSync(path.join(out,'xmpp-ui-result.json'),JSON.stringify({writes,errors,overflow,secretNeverReturned:true},null,2));
 console.log('XMPP_UI_PASS',JSON.stringify({writes,errors,overflow}));await browser.close();
})().catch(async e=>{console.error(e);if(page){console.log((await page.locator('body').innerText()).slice(-7000));await page.screenshot({path:path.join(out,'xmpp-ui-failure.png')});}if(browser)await browser.close();process.exit(1)});
