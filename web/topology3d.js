import * as THREE from './vendor/three.module.min.js';
import {OrbitControls} from './vendor/OrbitControls.js';

const host=document.getElementById('three-view'), model=window.topologyModel;
const button3D=document.getElementById('mode-3d'),button2D=document.getElementById('mode-2d');
let renderer,scene,camera,controls,group,labels,positions=new Map(),visible=false,needsFit=true,lastCount=0,view='overview',spacing=4,span=18;
const styles=getComputedStyle(document.documentElement);
const color=name=>styles.getPropertyValue('--'+name).trim();
const activeColor=color('green'),paper=color('paper'),orange=color('orange');
const records=new Map(),spines=new Map();let revision=-1;const reducedMotion=matchMedia('(prefers-reduced-motion: reduce)');
let motion=null,cameraMotion=null,frame=0,grid,spine,targets=new Map();
const raycaster=new THREE.Raycaster(),pointer=new THREE.Vector2();

function setup(){
 renderer=new THREE.WebGLRenderer({antialias:true,alpha:false});renderer.setPixelRatio(Math.min(devicePixelRatio,2));renderer.setClearColor(paper);renderer.domElement.tabIndex=0;renderer.domElement.setAttribute('aria-label','3D topology. Drag to orbit, scroll to zoom, right-drag to pan. Use the node buttons to select.');
 host.append(renderer.domElement);labels=document.createElement('div');labels.className='three-labels';host.append(labels);
 scene=new THREE.Scene();camera=new THREE.OrthographicCamera(-10,10,10,-10,.1,1000);controls=new OrbitControls(camera,renderer.domElement);controls.enableDamping=false;controls.minZoom=.3;controls.maxZoom=6;controls.maxPolarAngle=Math.PI*.88;
 scene.add(new THREE.HemisphereLight(0xffffff,0x687a64,2.5));const light=new THREE.DirectionalLight(0xffffff,3);light.position.set(-5,16,12);scene.add(light);
 group=new THREE.Group();scene.add(group);
 grid=new THREE.GridHelper(32,32,0xd7dfd3,0xe7ebe3);grid.position.y=-.65;grid.material.transparent=true;grid.material.opacity=.4;group.add(grid);
 spine=new THREE.Line(new THREE.BufferGeometry().setFromPoints([new THREE.Vector3(),new THREE.Vector3()]),new THREE.LineBasicMaterial({color:activeColor}));group.add(spine);
 controls.addEventListener('start',()=>{cameraMotion=null;});
 renderer.domElement.addEventListener('keydown',e=>{
 if(!['ArrowLeft','ArrowRight','ArrowUp','ArrowDown','+','=','-','f','F','Home'].includes(e.key))return;e.preventDefault();cameraMotion=null;
 if(e.key.toLowerCase()==='f')return frameSelection();if(e.key==='Home')return setView('overview');
 if(['+','=','-'].includes(e.key)){camera.zoom=THREE.MathUtils.clamp(camera.zoom*(e.key==='-'?.85:1.18),.3,6);camera.updateProjectionMatrix();return draw();}
 const offset=camera.position.clone().sub(controls.target),spherical=new THREE.Spherical().setFromVector3(offset);
 if(e.key==='ArrowLeft')spherical.theta-=.15;if(e.key==='ArrowRight')spherical.theta+=.15;
 if(e.key==='ArrowUp')spherical.phi-=.12;if(e.key==='ArrowDown')spherical.phi+=.12;
 spherical.phi=THREE.MathUtils.clamp(spherical.phi,.05,Math.PI*.85);camera.position.copy(controls.target).add(new THREE.Vector3().setFromSpherical(spherical));controls.update();draw();
 });
 controls.addEventListener('change',draw);
 let down;
 renderer.domElement.addEventListener('pointerdown',e=>{down={x:e.clientX,y:e.clientY};});
 renderer.domElement.addEventListener('pointerup',e=>{if(!down||Math.hypot(e.clientX-down.x,e.clientY-down.y)>5)return;const r=renderer.domElement.getBoundingClientRect();pointer.set((e.clientX-r.left)/r.width*2-1,-(e.clientY-r.top)/r.height*2+1);raycaster.setFromCamera(pointer,camera);const hit=raycaster.intersectObjects(group.children,true).find(h=>targets.has(h.object.userData.node)&&records.get(h.object.userData.node)?.object.visible);if(hit)model.select(hit.object.userData.node);});
 renderer.domElement.addEventListener('webglcontextlost',e=>{e.preventDefault();setMode(false);document.getElementById('toast').textContent='3D graphics context lost. The 2D topology is still available.';});
 new ResizeObserver(()=>{if(visible){resize();draw();}}).observe(host);
}
function createNode(n){
 const object=new THREE.Group(),isLayer=n.type==='layer';group.add(object);
 const geometry=isLayer?new THREE.BoxGeometry(3.5,.45,2.8):new THREE.BoxGeometry(.95,.65,.85);
 const material=new THREE.MeshStandardMaterial({color:activeColor,roughness:.8,transparent:true});
 const body=new THREE.Mesh(geometry,material);body.userData.node=n.id;object.add(body);
 const edge=new THREE.LineSegments(new THREE.EdgesGeometry(geometry),new THREE.LineBasicMaterial({color:0x70806e,transparent:true}));object.add(edge);
 const tinted=[material];
 if(isLayer)for(const x of [-1.1,0,1.1])for(const z of [-.8,.8]){const stud=new THREE.Mesh(new THREE.CylinderGeometry(.19,.19,.13,8),material);stud.position.set(x,.29,z);stud.userData.node=n.id;object.add(stud);}
 const platform=new THREE.Mesh(new THREE.BoxGeometry(isLayer?3.8:1.3,.09,isLayer?3.1:1.2),new THREE.MeshStandardMaterial({color:activeColor,transparent:true}));platform.position.y=isLayer?-.31:-.4;object.add(platform);
 const connection=n.parent?new THREE.Line(new THREE.BufferGeometry().setFromPoints(Array.from({length:4},()=>new THREE.Vector3())),new THREE.LineBasicMaterial({color:0x99aa96,transparent:true})):null;if(connection)group.add(connection);
 const label=document.createElement('button');label.dataset.node3d=n.id;label.dataset.object=object.uuid;label.append(document.createElement('strong'),document.createElement('small'));label.onclick=()=>model.select(n.id);
 const record={object,body,edge,platform,tinted,connection,label,node:n,opacity:0};records.set(n.id,record);return record;
}
function rebuild(){
 if(!renderer||!visible)return;
 const state=model.snapshot(),{layers,branches,selected,focused,layout}=state;
 if(revision!==state.revision){for(const r of records.values()){const disposed=new Set();r.object.traverse(o=>{o.geometry?.dispose();if(o.material&&!disposed.has(o.material)){o.material.dispose();disposed.add(o.material);}});r.connection?.geometry.dispose();r.connection?.material.dispose();group.remove(r.object);if(r.connection)group.remove(r.connection);r.label.remove();}records.clear();for(const line of spines.values()){line.geometry.dispose();line.material.dispose();group.remove(line);}spines.clear();motion=null;cameraMotion=null;revision=state.revision;needsFit=true;view='overview';}
 const stackIds=state.stackIds||['app'],stackOf=n=>n.stackId||'app',stackLayers=n=>layers.filter(l=>stackOf(l)===stackOf(n));
 const firstBuild=records.size===0,count=layers.length+branches.length;if(count!==lastCount){needsFit=true;lastCount=count;}
 const all=[...layers,...branches],get=id=>all.find(n=>n.id===id),path=id=>{const a=[];for(let n=get(id);n;n=get(n.parent))a.unshift(n);return a;};
 const selectedPath=new Set(path(selected).map(n=>n.id)),peers=new Set();for(const l of layers)if(l.source&&(l.id===selected||l.source===selected)){peers.add(l.id);peers.add(l.source);for(const n of path(l.source))selectedPath.add(n.id);}
 const visiblePositions=new Map(layout.positions),baseId=path(selected)[0].id,baseIndex=stackLayers(get(baseId)).findIndex(l=>l.id===baseId);targets=new Map();
 const columns=Math.ceil(Math.sqrt(stackIds.length)),tileX=12+Math.max(1,...branches.map(b=>path(b.id).length-1))*5,tileZ=Math.max(20,...layers.map(l=>branches.filter(b=>path(b.id)[0].id===l.id).length*3+8));
 for(const [s,stack] of stackIds.entries()){let y=0;const local=layers.filter(l=>stackOf(l)===stack);for(const [i,layer] of local.entries()){if(i)y+=(stack===stackOf(get(baseId))&&(i===baseIndex||i-1===baseIndex))?spacing+1.3:Math.max(2.3,spacing*.72);targets.set(layer.id,new THREE.Vector3((s%columns)*tileX,y,Math.floor(s/columns)*tileZ));}}
 for(const branch of branches){if(!visiblePositions.has(branch.id))continue;const chain=path(branch.id),base=chain[0],root=chain[1],siblings=branches.filter(n=>n.parent===base.id),sign=siblings.findIndex(n=>n.id===root.id)%2?-1:1;
 targets.set(branch.id,new THREE.Vector3(targets.get(base.id).x+sign*(2.1+(chain.length-1)*2.45),targets.get(base.id).y+.3,targets.get(base.id).z+(visiblePositions.get(branch.id).y-visiblePositions.get(base.id).y)/94*2.8));}
 if(view==='layer')for(const id of [...targets.keys()])if(path(id)[0].id!==baseId)targets.delete(id);
 spine.visible=false;for(const stack of stackIds){if(!spines.has(stack)){const line=new THREE.Line(new THREE.BufferGeometry().setFromPoints([new THREE.Vector3(),new THREE.Vector3()]),new THREE.LineBasicMaterial({color:activeColor}));spines.set(stack,line);group.add(line);}spines.get(stack).visible=view!=='layer';}grid.position.y=view==='layer'?targets.get(baseId).y-.45:-.65;
 const moves=[];positions=new Map();
 for(const n of all){const target=targets.get(n.id);let r=records.get(n.id);if(!r&&!target)continue;
 const created=!r;r??=createNode(n);
 if(created){r.object.position.copy(target);if(!firstBuild){if(n.type==='layer'){const previous=records.get(stackLayers(n).at(-2)?.id);if(previous)r.object.position.y=previous.object.position.y+.7;}else{const parent=records.get(n.parent);if(parent)r.object.position.copy(parent.object.position);}}}
 const isSelected=n.id===selected,peer=peers.has(n.id)&&!isSelected,otherTree=n.type==='branch'&&path(n.id)[0].id!==baseId;
 const dim=focused&&!selectedPath.has(n.id)&&!path(n.id).some(a=>a.id===selected);
 const opacity=!target?0:dim?.16:otherTree&&!peers.has(n.id)?.42:1;
 const tint=new THREE.Color(peer?orange:isSelected?activeColor:n.type==='layer'?0xc4d4b6:[0x86aa8b,0xd5b889,0x8fadc1][layers.findIndex(l=>l.id===path(n.id)[0].id)%3]);
 r.platform.visible=isSelected;r.edge.material.color.set(isSelected?activeColor:0x70806e);if(r.connection)r.connection.material.color.set(selectedPath.has(n.id)?activeColor:0x99aa96);
 r.label.className='three-node '+n.type+(isSelected?' selected':'')+(peer?' promotion-peer':'');r.label.setAttribute('aria-pressed',String(isSelected));r.label.title=n.name;r.label.firstChild.textContent=(n.type==='layer'&&stackIds.length>1?stackOf(n)+' / ':'')+n.name+(n.type==='layer'&&n.id===stackLayers(n).at(-1).id?' · HEAD':'');
 const promoted=layers.filter(l=>l.source===n.id);r.label.lastChild.textContent=n.type==='layer'?(n.source?'From '+n.source:'Genesis'):promoted.length?'Promoted → '+promoted.map(l=>l.id).join(', '):'Branch';r.label.lastChild.className=n.source||promoted.length?'promotion-text':'';
 if(target){positions.set(n.id,r.object.position);if(!r.label.isConnected)labels.append(r.label);}else r.label.remove();
 moves.push({r,from:r.object.position.clone(),to:target||r.object.position.clone(),opacity:r.opacity,toOpacity:opacity,color:r.body.material.color.clone(),toColor:tint});
 }
 motion={start:performance.now(),duration:firstBuild||reducedMotion.matches?0:460,moves};
 updateNavigation(state);resize();if(needsFit){fit(null,!firstBuild);needsFit=false;}schedule();
}
function schedule(){if(visible&&!document.hidden&&!frame){host.dataset.animating='true';frame=requestAnimationFrame(tick);}}
function tick(now){
 frame=0;if(!visible)return;
 if(motion){const t=motion.duration?Math.min(1,(now-motion.start)/motion.duration):1,e=1-Math.pow(1-t,3);
 for(const m of motion.moves){const r=m.r;r.object.position.lerpVectors(m.from,m.to,e);r.opacity=THREE.MathUtils.lerp(m.opacity,m.toOpacity,e);r.object.visible=r.opacity>.005;r.body.material.color.lerpColors(m.color,m.toColor,e);r.body.material.opacity=r.opacity;r.edge.material.opacity=r.opacity*.7;r.platform.material.opacity=r.opacity;r.label.style.opacity=String(r.opacity);if(r.connection){r.connection.visible=r.opacity>.005;r.connection.material.opacity=r.opacity;}}
 if(t===1)motion=null;}
 if(cameraMotion){const m=cameraMotion,t=m.duration?Math.min(1,(now-m.start)/m.duration):1,e=t*t*(3-2*t);
 camera.position.lerpVectors(m.from,m.to,e);controls.target.lerpVectors(m.fromTarget,m.toTarget,e);span=THREE.MathUtils.lerp(m.fromSpan,m.toSpan,e);camera.zoom=THREE.MathUtils.lerp(m.fromZoom,1,e);resize();controls.update();if(t===1)cameraMotion=null;}
 draw();host.dataset.animating=String(Boolean(motion||cameraMotion));if(motion||cameraMotion)schedule();
}
function updateConnections(){
 for(const r of records.values())if(r.connection&&r.connection.visible){const parent=records.get(r.node.parent);if(!parent)continue;const p=r.object.position,start=parent.object.position.clone();if(parent.node.type==='layer')start.x+=Math.sign(p.x-start.x)*1.75;
 const points=[start,new THREE.Vector3((start.x+p.x)/2,start.y,start.z),new THREE.Vector3((start.x+p.x)/2,p.y,p.z),p],attribute=r.connection.geometry.attributes.position;points.forEach((v,i)=>attribute.setXYZ(i,v.x,v.y,v.z));attribute.needsUpdate=true;r.connection.geometry.computeBoundingSphere();}
 const layers=model.snapshot().layers;for(const [stack,line] of spines){const local=layers.filter(l=>(l.stackId||'app')===stack),first=records.get(local[0]?.id),last=records.get(local.at(-1)?.id);if(first&&last){const a=line.geometry.attributes.position;a.setXYZ(0,...first.object.position.toArray());a.setXYZ(1,...last.object.position.toArray());a.needsUpdate=true;line.geometry.computeBoundingSphere();}}
}
function resize(){if(!renderer||!host.clientWidth)return;renderer.setSize(host.clientWidth,host.clientHeight,false);const aspect=host.clientWidth/host.clientHeight;camera.left=-span*aspect/2;camera.right=span*aspect/2;camera.top=span/2;camera.bottom=-span/2;camera.updateProjectionMatrix();}
function draw(){
 if(!visible||!renderer)return;updateConnections();renderer.render(scene,camera);
 const occupied=[],selected=model.snapshot().selected;
 const projected=[...labels.children].map(label=>{const point=positions.get(label.dataset.node3d).clone();point.y+=label.classList.contains('layer')?.6:.6;point.project(camera);return {label,point,x:(point.x*.5+.5)*host.clientWidth,y:(-point.y*.5+.5)*host.clientHeight};}).sort((a,b)=>Number(b.label.dataset.node3d===selected)-Number(a.label.dataset.node3d===selected)||Number(b.label.classList.contains('layer'))-Number(a.label.classList.contains('layer'))||Number(b.label.classList.contains('promotion-peer'))-Number(a.label.classList.contains('promotion-peer'))||a.y-b.y);
 // Keep labels anchored. The node picker and Layer view expose crowded branches.
 for(const {label,point,x,y} of projected){label.hidden=false;const w=label.offsetWidth,h=label.offsetHeight;
 const candidates=[{x:x-w/2,y:y-h,w,h},{x:x-w/2,y:y+16,w,h}];
 const free=r=>!occupied.some(o=>r.x<o.x+o.w+4&&r.x+w+4>o.x&&r.y<o.y+o.h+4&&r.y+h+4>o.y);
 const rect=candidates.find(free)||candidates[0],collision=!free(rect);
 label.hidden=point.z>1||point.z< -1||rect.x<0||rect.x+w>host.clientWidth||rect.y<0||y>host.clientHeight||collision;
 if(label.hidden)continue;occupied.push(rect);label.style.left=rect.x+'px';label.style.top=rect.y+'px';label.style.zIndex=String(Math.round((1-point.z)*1000));
 }
}
function fit(ids=null,animate=true){
 if(!renderer||!positions.size)return;
 const points=ids?ids.map(id=>targets.get(id)).filter(Boolean):[...targets.values()];if(!points.length)return;
 const box=new THREE.Box3().setFromPoints(points).expandByScalar(1.8),center=box.getCenter(new THREE.Vector3());
 const direction=view==='layer'?new THREE.Vector3(0,1,.001):view==='stack'?new THREE.Vector3(0,.05,1):new THREE.Vector3(1,.9,1.4);
 const to=center.clone().add(direction.normalize().multiplyScalar(70)),probe=camera.clone();probe.position.copy(to);probe.lookAt(center);probe.updateMatrixWorld();
 const inverse=probe.quaternion.clone().invert();let maxX=0,maxY=0;
 for(const x of [box.min.x,box.max.x])for(const y of [box.min.y,box.max.y])for(const z of [box.min.z,box.max.z]){const v=new THREE.Vector3(x,y,z).sub(center).applyQuaternion(inverse);maxX=Math.max(maxX,Math.abs(v.x));maxY=Math.max(maxY,Math.abs(v.y));}
 cameraMotion={start:performance.now(),duration:animate&&!reducedMotion.matches?480:0,from:camera.position.clone(),to,fromTarget:controls.target.clone(),toTarget:center,fromSpan:span,toSpan:Math.max(maxY*2,maxX*2/(host.clientWidth/host.clientHeight))*1.16,fromZoom:camera.zoom};schedule();
}
function frameSelection(){const s=model.snapshot(),n=[...s.layers,...s.branches].find(n=>n.id===s.selected);fit([n.id,n.parent,...s.branches.filter(b=>b.parent===n.id).map(b=>b.id)].filter(Boolean));}
function setView(next){view=next;needsFit=true;rebuild();}
function updateNavigation(state){
 for(const button of document.querySelectorAll('[data-camera]'))button.setAttribute('aria-pressed',String(button.dataset.camera===view));
 const all=[...state.layers,...state.branches],path=[];for(let n=all.find(n=>n.id===state.selected);n;n=all.find(p=>p.id===n.parent))path.unshift(n);
 const crumbs=document.getElementById('three-breadcrumbs');crumbs.replaceChildren();for(const n of path){const b=document.createElement('button');b.textContent=n.name;b.onclick=()=>model.select(n.id);crumbs.append(b);}
 const n=path.at(-1),siblings=n.type==='layer'?state.layers.filter(l=>(l.stackId||'app')===(n.stackId||'app')):state.branches.filter(b=>b.parent===n.parent),index=siblings.findIndex(b=>b.id===n.id);
 document.getElementById('nav-parent').disabled=!n.parent;document.getElementById('nav-parent').onclick=()=>model.select(n.parent);
 for(const [id,offset] of [['nav-prev',-1],['nav-next',1]]){const button=document.getElementById(id);button.disabled=!siblings[index+offset];button.onclick=()=>model.select(siblings[index+offset].id);}
}
function setMode(use3D){
 try{if(use3D&&!renderer)setup();visible=use3D;window.topology3D.visible=visible;host.hidden=!visible;document.getElementById('extent').hidden=visible;document.getElementById('viewport').classList.toggle('is-3d',visible);button3D.setAttribute('aria-pressed',String(visible));button2D.setAttribute('aria-pressed',String(!visible));document.getElementById('zoom-label').hidden=visible;document.getElementById('three-help').hidden=!visible;document.getElementById('three-picker-wrap').hidden=!visible;document.getElementById('three-navigation').hidden=!visible;
 if(visible){needsFit=true;rebuild();}else{cancelAnimationFrame(frame);frame=0;cameraMotion=null;motion=null;model.refresh();}
 }catch(error){console.error(error);visible=false;window.topology3D.visible=false;host.hidden=true;document.getElementById('extent').hidden=false;document.getElementById('viewport').classList.remove('is-3d');button3D.setAttribute('aria-pressed','false');button2D.setAttribute('aria-pressed','true');document.getElementById('three-help').hidden=true;document.getElementById('three-picker-wrap').hidden=true;document.getElementById('three-navigation').hidden=true;document.getElementById('toast').textContent='3D is unavailable in this browser. You can continue in 2D.';}
}
window.topology3D={visible:false};
button3D.onclick=()=>setMode(true);button2D.onclick=()=>setMode(false);
for(const [id,factor] of [['zoom-in',.85],['zoom-out',1.18],['fit',null]]){const button=document.getElementById(id),original=button.onclick;button.onclick=()=>{if(!visible)return original();if(factor){cameraMotion=null;camera.zoom=THREE.MathUtils.clamp(camera.zoom/factor,.3,6);camera.updateProjectionMatrix();draw();}else fit();};}
document.querySelectorAll('[data-camera]').forEach(button=>button.onclick=()=>setView(button.dataset.camera));
 document.getElementById('frame-selected').onclick=frameSelection;
 document.getElementById('layer-spacing').oninput=e=>{spacing=Number(e.target.value);needsFit=true;rebuild();};
 document.getElementById('three-picker').onchange=e=>model.select(e.target.value);
window.addEventListener('topology-change',()=>{const picker=document.getElementById('three-picker'),state=model.snapshot();picker.replaceChildren(...[...state.layers,...state.branches].map(n=>{const option=document.createElement('option');option.value=n.id;option.textContent=n.type+' · '+n.id;return option;}));picker.value=state.selected;if(view==='layer')needsFit=true;rebuild();});
document.addEventListener('visibilitychange',()=>{if(document.hidden){cancelAnimationFrame(frame);frame=0;}else if(visible)schedule();});
reducedMotion.addEventListener('change',()=>{if(reducedMotion.matches){if(motion)motion.duration=0;if(cameraMotion)cameraMotion.duration=0;schedule();}});
model.refresh();setMode(true);
