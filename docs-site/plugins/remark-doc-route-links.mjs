import { existsSync, readFileSync, realpathSync } from 'node:fs';
import { dirname, relative, resolve, sep, basename, extname } from 'node:path';
import { fileURLToPath } from 'node:url';

const defaultDocsRoot = fileURLToPath(new URL('../src/content/docs/', import.meta.url));
const routed = new Set(['primary', 'reference']);
const sourceMarkdown = /\.(?:md|mdx)$/i;
const external = /^(?:[a-z][a-z\d+.-]*:|\/\/|\/)/i;

function routeForSource(root, path) {
  let suffix = relative(root, path).split(sep).join('/');
  if (suffix === 'index.mdx') return '/';
  if (basename(path) === 'README.md') suffix = suffix.slice(0, -'README.md'.length) + 'readme';
  else if (basename(path) === 'index.md') suffix = suffix.slice(0, -'index.md'.length).replace(/\/$/, '');
  else suffix = suffix.slice(0, -extname(suffix).length);
  return `/${suffix.replace(/^\/+|\/+$/g, '')}/`.replace('//', '/');
}

function publication(path) {
  const frontmatter = readFileSync(path, 'utf8').match(/^---\n([\s\S]*?)\n---/);
  return frontmatter?.[1].match(/^publication:\s*([\w-]+)/m)?.[1];
}

/** Resolve source-document links to their published Starlight route without changing source Markdown. */
export function expandDocumentationLinks(tree, file, docsRoot = defaultDocsRoot) {
  if (file?.data?.astro?.frontmatter?.publication && !routed.has(file.data.astro.frontmatter.publication)) return;
  const root = realpathSync(docsRoot);
  let source;
  function transform(node) {
    if (['link', 'definition'].includes(node.type) && typeof node.url === 'string') {
      const url = node.url;
      const match = /^([^?#]+)(\?[^#]*)?(#.*)?$/.exec(url);
      if (match && sourceMarkdown.test(match[1]) && !external.test(match[1])) {
        if (!source) {
          const filePath = file?.path;
          if (!filePath) throw new Error(`Cannot resolve source-relative documentation link without Markdown file path: ${url}`);
          source = realpathSync(filePath);
          if (!source.startsWith(root + sep)) throw new Error(`Markdown source outside docs root: ${source}`);
        }
        const target = resolve(dirname(source), decodeURIComponent(match[1]));
        if (!existsSync(target)) throw new Error(`Broken source-document link in ${source}: ${url}`);
        const canonical = realpathSync(target);
        if (!canonical.startsWith(root + sep)) throw new Error(`Documentation link escapes docs root in ${source}: ${url}`);
        if (!routed.has(publication(canonical))) throw new Error(`Documentation link points to non-routed content in ${source}: ${url}`);
        node.url = `/runenwerk${routeForSource(root, canonical)}${match[2] || ''}${match[3] || ''}`;
      }
    }
    for (const child of node.children ?? []) transform(child);
  }
  transform(tree);
}

export function remarkDocumentationLinks() {
  return (tree, file) => expandDocumentationLinks(tree, file);
}
