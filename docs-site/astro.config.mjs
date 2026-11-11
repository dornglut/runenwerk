// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import { remarkRustExampleSource } from './plugins/remark-rust-example-source.mjs';

// https://astro.build/config
export default defineConfig({
	site: 'https://dornglut.github.io/runenwerk',
	base: '/runenwerk',
	markdown: {
		remarkPlugins: [remarkRustExampleSource],
	},
	integrations: [
		starlight({
			title: 'Runenwerk Docs',
			sidebar: [
				{
					label: 'Start Here',
					items: [
						{ label: 'Overview', link: '/' },
						{ label: 'Build an Application', slug: 'engine/reference/usage-guide' },
						{ label: 'Explore Examples', slug: 'engine/examples/overview' },
						{
							label: 'Understand Runenwerk',
							slug: 'architecture/runenwerk-platform-architecture',
						},
					],
				},
				{
					label: 'Primary documentation',
					items: [
						{ label: 'Adapters', autogenerate: { directory: 'adapters' } },
						{ label: 'Apps', autogenerate: { directory: 'apps' } },
						{ label: 'Domain', autogenerate: { directory: 'domain' } },
						{ label: 'Engine', autogenerate: { directory: 'engine' } },
						{ label: 'Foundation', autogenerate: { directory: 'foundation' } },
						{ label: 'Net', autogenerate: { directory: 'net' } },
					],
				},
				{
					label: 'Reference',
					items: [
						{ label: 'Architecture', autogenerate: { directory: 'architecture' } },
						{ label: 'ADRs', autogenerate: { directory: 'adr' } },
						{ label: 'Design', autogenerate: { directory: 'design' } },
						{ label: 'Guidelines', autogenerate: { directory: 'guidelines' } },
						{ label: 'Reports', autogenerate: { directory: 'reports' } },
					],
				},
			],
		}),
	],
});
