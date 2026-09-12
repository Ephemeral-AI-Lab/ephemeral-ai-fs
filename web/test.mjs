// Run: node web/test.mjs — no dependencies.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import vm from 'node:vm';
const html=readFileSync(new URL('./index.html',import.meta.url),'utf8');
for(const script of html.matchAll(/<script[^>]*>([\s\S]*?)<\/script>/g))new vm.Script(script[1]);
const context=vm.createContext({assert});
vm.runInContext(html.match(/<script id="model">([\s\S]*?)<\/script>/)[1],context);
vm.runInContext(`
assert.equal(ancestry('refine').map(n=>n.id).join(','),'L2,search-a,narrow,refine');
assert.equal(ancestry('L2').length,1,'Layer ancestry should not enter the branch path');
assert.equal(descendants('search-a').length,3);
fork('from-layer','L1');
fork('from-branch','from-layer');
assert.equal(ancestry('from-branch').map(n=>n.id).join(','),'L1,from-layer,from-branch');
assert.throws(()=>fork('from-layer','L2'),/already/);
assert.throws(()=>fork('<bad>','L2'),/lowercase/);
assert.throws(()=>fork('valid','missing'),/does not exist/);
assert.equal(layers[0].source,null);
for(const [i,layer] of layers.entries())if(i){
 assert.equal(getNode(layer.source).type,'branch','Every non-genesis layer must have a source branch');
 assert.equal(ancestry(layer.source)[0].id,layers[i-1].id,'Promotion must come from the preceding base layer');
}
assert.throws(()=>addToStack('L3'),/Select a branch/);
assert.throws(()=>addToStack('missing'),/Select a branch/);
assert.throws(()=>addToStack('wide'),/stack head is L3/);
const forkSource=getNode('patch').commits.length-1;
const child=fork('snapshot-test','patch');
assert.equal(child.commits.length,1);
assert.equal(child.commits[0].number,0);
assert.equal(child.sourceCommit,forkSource);
assert.equal(JSON.stringify(child.commits[0].files),JSON.stringify(getNode('patch').commits[forkSource].files));
const forkFiles=JSON.stringify(child.commits[0].files);
appendCommit('patch');
assert.equal(child.sourceCommit,forkSource);
assert.equal(JSON.stringify(child.commits[0].files),forkFiles,'Fork state must remain pinned');
appendCommit(child.id);
assert.equal(child.commits[1].number,1);
assert.equal(JSON.stringify(child.commits[0].files),forkFiles,'Later changes must not mutate Commit 0');
assert.equal(layers[2].sourceCommit,2);
const branchCount=branches.length;
const promoted=addToStack('minimal');
assert.equal(promoted.id,'L4');
assert.equal(promoted.source,'minimal');
assert.equal(layers.at(-1),promoted);
assert.equal(branches.length,branchCount,'Promotion preserves the branch tree');
assert.equal(addToStack('minimal'),promoted,'Repeated Add must not create another layer');
const promotedCommit=promoted.sourceCommit, layerSnapshot=JSON.stringify(layerFiles(promoted.id));
appendCommit('minimal');
assert.equal(promoted.sourceCommit,promotedCommit);
assert.equal(JSON.stringify(layerFiles(promoted.id)),layerSnapshot,'Promotion must stay pinned to its original commit');
assert.throws(()=>addToStack('minimal'),/stack head is L4/);
assert.throws(()=>addToStack('alternate'),/stack head is L4/);
fork('next-layer','L4');
assert.equal(addToStack('next-layer').id,'L5');
assert.ok(layout().positions.get('L5').y<layout().positions.get('L4').y);
const full=layout(), hidden=layout(new Set(['search-a']));
assert.ok(full.positions.get('L3').y<full.positions.get('L2').y);
assert.ok(full.positions.get('L2').y<full.positions.get('L1').y);
assert.ok(full.positions.has('refine'));
assert.ok(!hidden.positions.has('refine'));
assert.ok(hidden.positions.has('search-a'));
assert.ok(hidden.positions.has('search-b'));
for(const n of branches){const p=full.positions.get(n.id),parent=full.positions.get(n.parent);assert.ok(p.x>parent.x);}
const leaves=branches.filter(n=>!children(n.id).length).map(n=>full.positions.get(n.id).y);
assert.equal(new Set(leaves).size,leaves.length,'Leaf rows must not collide');

const options={layerCount:3,branchCount:4,depth:3,seed:42};
generateScene(options);
assert.equal(stackIds().length,1);assert.equal(layers.length,3);assert.equal(branches.length,12);
generateScene({...options,layerCount:1,branchCount:100});assert.equal(layers.length,1);assert.equal(branches.length,100);assert.throws(()=>generateScene({...options,branchCount:101}),/ranges/);
generateScene(options);
const snapshot=JSON.stringify([layers,branches]);
generateScene(options);assert.equal(JSON.stringify([layers,branches]),snapshot,'Same seed must reproduce scene');
for(const b of branches){assert.ok(ancestry(b.id).length-1<=3);assert.equal(stackOf(b.id),stackOf(b.parent));assert.equal(b.commits[0].number,0);}
for(const l of layers)if(l.source){assert.equal(stackOf(l.id),stackOf(l.source));assert.ok(getNode(l.source).commits[l.sourceCommit]);}
assert.equal(addToStack('app-l3-b1').id,'L4');
const beforeInvalid=JSON.stringify([layers,branches]);assert.throws(()=>generateScene({...options,layerCount:101}),/ranges/);assert.equal(JSON.stringify([layers,branches]),beforeInvalid);
generateScene({...options,layerCount:100});assert.equal(layers.length,100);assert.equal(stackIds().length,1);
generateScene({...options,stacks:100});assert.equal(stackIds().length,1,'Legacy stack count must not create multiple projects');
generateScene({...options,depth:1});assert.ok(branches.every(b=>getNode(b.parent).type==='layer'));
`,context);
console.log('PASS: syntax, layer/branch forks, provenance, validation, subtree collapse, newest-first order, promotion provenance, Add to stack, repeat-add safety, and tree layout.');
