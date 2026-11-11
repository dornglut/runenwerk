import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, rmSync, symlinkSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, dirname } from 'node:path';
import { test } from 'node:test';
import { expandDocumentationLinks } from '../plugins/remark-doc-route-links.mjs';

const page = (kind='primary') => `---\ntitle: Example\npublication: ${kind}\n---\n\n# Example\n`;
function fixture(t) {
  const root = mkdtempSync(join(tmpdir(), 'runenwerk-remark-route-'));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const write = (path, content) => {
    const full = join(root, path);
    mkdirSync(dirname(full), { recursive: true });
    writeFileSync(full, content);
    return full;
  };
  const source = write('engine/examples/overview.md', page());
  write('engine/README.md', page());
  write('engine/plugins/README.md', page());
  write('architecture/platform.md', page('reference'));
  write('workspace/README.md', page('repository-current'));
  return { root, source, write };
}
const tree = urls => ({type: 'root', children: urls.map(url=>({type: 'paragraph',children:[{type: 'link',url}]}))});
const links = ast => ast.children.map(x=>x.children[0].url);

test('converts source .md links to correct routes including README, root, query, fragment', t => {
 const {root,source}=fixture(t);
 const ast=tree(['../README.md','../plugins/README.md#usage','../../architecture/platform.md?q=1#overview','https://github.com/example.md','./overview.md','../plugins/']);
 expandDocumentationLinks(ast,{path:source},root);
 assert.deepEqual(links(ast),[
  '/runenwerk/engine/readme/', '/runenwerk/engine/plugins/readme/#usage',
  '/runenwerk/architecture/platform/?q=1#overview','https://github.com/example.md',
  '/runenwerk/engine/examples/overview/', '../plugins/'
 ]);
});

test('keeps source-only docs non-routed and fails closed on missing destinations', t => {
 const {root,source}=fixture(t);
 for(const url of ['../../workspace/README.md','../missing.md','../../../README.md']){
  assert.throws(()=>expandDocumentationLinks(tree([url]),{path:source},root), Error,url);
 }
});

test('requires source path for a relative Markdown URL and preserves unrelated nodes', t=>{
 const {root,source}=fixture(t);
 assert.throws(()=>expandDocumentationLinks(tree(['../README.md']),{},root),/without Markdown file path/);
 const ast=tree(['https://example.com/foo.md','#readme','/runenwerk/engine/readme/']);
 expandDocumentationLinks(ast,{path:source},root);
 assert.deepEqual(links(ast),['https://example.com/foo.md','#readme','/runenwerk/engine/readme/']);
});

test('rejects a symlink to a Markdown file outside docs root', {skip:process.platform === 'win32'}, t=>{
 const {root,source}=fixture(t);
 const outside = mkdtempSync(join(tmpdir(), 'runenwerk-remark-external-'));
 t.after(()=>rmSync(outside,{recursive:true,force:true}));
 const file=join(outside,'external.md');writeFileSync(file,page());
 symlinkSync(file,join(root,'engine','plugins','escaped.md'));
 assert.throws(()=>expandDocumentationLinks(tree(['../plugins/escaped.md']),{path:source},root),/escapes docs root/);
});

test('does not reinterpret retained history or repository-current sources', t=>{
 const {root,source}=fixture(t);
 const ast=tree(['../../../old/retained.md']);
 expandDocumentationLinks(ast,{path:source,data:{astro:{frontmatter:{publication:'history'}}}},root);
 assert.deepEqual(links(ast),['../../../old/retained.md']);
});
