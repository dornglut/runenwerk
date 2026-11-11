import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../../', import.meta.url));
const compact = (text) => text.replace(/[\s\u200b]/g, '');

function decodeEntities(text) {
  const named = { amp: '&', lt: '<', gt: '>', quot: '"', apos: "'", nbsp: ' ' };
  return text.replace(/&(#x[0-9a-f]+|#\d+|amp|lt|gt|quot|apos|nbsp);/gi, (_, entity) => {
    if (entity.startsWith('#x')) return String.fromCodePoint(parseInt(entity.slice(2), 16));
    if (entity.startsWith('#')) return String.fromCodePoint(parseInt(entity.slice(1), 10));
    return named[entity.toLowerCase()];
  });
}

for (const [slug, example] of [
  ['runtime-minimal', 'runtime_minimal'],
  ['window-input-demo', 'window_input_demo'],
]) {
  const source = readFileSync(join(root, 'engine/examples', example, 'main.rs'), 'utf8');
  const html = readFileSync(join(root, 'docs-site/dist/engine/examples', slug, 'readme/index.html'), 'utf8');
  const codeBlocks = [...html.matchAll(/<pre\b[^>]*>\s*<code\b[^>]*>([\s\S]*?)<\/code>\s*<\/pre>/gi)]
    .map((match) => compact(decodeEntities(match[1].replace(/<[^>]*>/g, ''))));

  assert.ok(codeBlocks.some((block) => block.includes(compact(source))),
    `Built ${slug} page does not contain the complete Rust source of ${example}/main.rs`);
  console.log(`PASS: ${slug} renders complete ${example}/main.rs source`);
}
