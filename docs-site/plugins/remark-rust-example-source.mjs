import { readFileSync, realpathSync, statSync } from 'node:fs';
import { resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

const repositoryRoot = fileURLToPath(new URL('../../', import.meta.url));
const sourceDirective = /^runenwerk-source=(engine\/examples\/[a-z0-9_]+\/[a-z0-9_]+\.rs)$/;

/** Insert maintained Rust example files into ordinary Markdown code fences. */
export function expandRustExampleSource(tree, root = repositoryRoot) {
  function visit(node) {
    if (node.type === 'code' && typeof node.meta === 'string' && node.meta.includes('runenwerk-source')) {
      const match = sourceDirective.exec(node.meta);
      if (!match || node.lang !== 'rust' || (node.value ?? '').trim() !== '') {
        throw new Error(`Invalid Runenwerk example source directive: ${node.meta}`);
      }

      // Resolve symlinks as well as '..' to prevent reading arbitrary files.
      const permittedRoot = realpathSync(resolve(root, 'engine', 'examples'));
      const source = realpathSync(resolve(root, match[1]));
      if (!source.startsWith(permittedRoot + sep) || !statSync(source).isFile()) {
        throw new Error(`Rust example source is outside engine/examples: ${match[1]}`);
      }
      const content = readFileSync(source, 'utf8');
      if (!content.trim()) {
        throw new Error(`Rust example source is empty: ${match[1]}`);
      }
      node.value = content.replace(/\n$/, '');
      node.meta = null;
    }
    for (const child of node.children ?? []) visit(child);
  }
  visit(tree);
}

export function remarkRustExampleSource() {
  return (tree) => expandRustExampleSource(tree);
}
