import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, readFileSync, rmSync, symlinkSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { test } from 'node:test';
import { fileURLToPath } from 'node:url';
import { expandRustExampleSource } from '../plugins/remark-rust-example-source.mjs';

function fixture(t) {
  const root = mkdtempSync(join(tmpdir(), 'runenwerk-source-docs-'));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  mkdirSync(join(root, 'engine', 'examples', 'minimal'), { recursive: true });
  writeFileSync(join(root, 'engine', 'examples', 'minimal', 'main.rs'), 'fn main() { println!("real source"); }\n');
  return root;
}

function document(meta, lang = 'rust', value = '') {
  return { type: 'root', children: [{ type: 'blockquote', children: [{ type: 'code', lang, meta, value }] }] };
}

const directive = 'runenwerk-source=engine/examples/minimal/main.rs';

test('loads real source unchanged and removes the source-only directive', (t) => {
  const root = fixture(t);
  const tree = document(directive);
  expandRustExampleSource(tree, root);
  const code = tree.children[0].children[0];
  assert.equal(code.value, 'fn main() { println!("real source"); }');
  assert.equal(code.meta, null);
  assert.equal(code.lang, 'rust');
});

test('does not change ordinary code blocks', (t) => {
  const root = fixture(t);
  const tree = document('title=example.rs', 'rust', 'let value = 2;');
  expandRustExampleSource(tree, root);
  assert.equal(tree.children[0].children[0].value, 'let value = 2;');
});

test('rejects missing, traversal, malformed, and non-Rust directives', (t) => {
  const root = fixture(t);
  for (const meta of [
    'runenwerk-source=engine/examples/missing/main.rs',
    'runenwerk-source=../../etc/passwd',
    'runenwerk-source=engine/examples/minimal/main.rs extra=1',
    'runenwerk-source=engine/examples/minimal/main.rs.txt',
  ]) {
    assert.throws(() => expandRustExampleSource(document(meta), root), Error, meta);
  }
  assert.throws(() => expandRustExampleSource(document(directive, 'text'), root));
  assert.throws(() => expandRustExampleSource(document(directive, 'rust', 'old duplicated code'), root));
});

test('rejects an example symlink escaping the repository source tree', { skip: process.platform === 'win32' }, (t) => {
  const root = fixture(t);
  const outside = mkdtempSync(join(tmpdir(), 'runenwerk-source-outside-'));
  t.after(() => rmSync(outside, { recursive: true, force: true }));
  writeFileSync(join(outside, 'external.rs'), 'fn main() {}\n');
  symlinkSync(join(outside, 'external.rs'), join(root, 'engine', 'examples', 'minimal', 'external.rs'));
  assert.throws(() => expandRustExampleSource(document('runenwerk-source=engine/examples/minimal/external.rs'), root), /outside engine\/examples/);
});

test('actual pilot pages reference registered, maintained Cargo examples', () => {
  const root = fileURLToPath(new URL('../../', import.meta.url));
  const cargo = readFileSync(join(root, 'engine', 'Cargo.toml'), 'utf8');
  for (const [slug, rust] of [
    ['runtime-minimal', 'runtime_minimal'],
    ['window-input-demo', 'window_input_demo'],
  ]) {
    const filename = `engine/examples/${rust}/main.rs`;
    const page = readFileSync(join(root, 'docs-site/src/content/docs/engine/examples', slug, 'README.md'), 'utf8');
    assert.ok(page.includes('```rust runenwerk-source=' + filename + '\n```'), slug);
    assert.ok(cargo.includes('name = "' + rust + '"\npath = "examples/' + rust + '/main.rs"'), rust);
    assert.ok(readFileSync(resolve(root, filename), 'utf8').length > 0, filename);
  }
});
