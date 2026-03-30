// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

// https://astro.build/config
export default defineConfig({
	integrations: [
		starlight({
			title: 'Runenwerk Docs',
			social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/withastro/starlight' }],
			sidebar: [
				{
					label: 'Workspace',
					autogenerate: { directory: 'workspace' },
				},
				{
					label: 'Domain',
					autogenerate: { directory: 'domain' },
				},
				{
					label: 'Engine',
					autogenerate: { directory: 'engine' },
				},
				{
					label: 'Net',
					autogenerate: { directory: 'net' },
				},
				{
					label: 'Games',
					autogenerate: { directory: 'games' },
				},
				{
					label: 'Apps',
					autogenerate: { directory: 'apps' },
				},
				{
					label: 'Assets',
					autogenerate: { directory: 'assets' },
				},
				{
					label: 'Reference',
					autogenerate: { directory: 'reference' },
				},
				{
					label: 'Guides',
					autogenerate: { directory: 'guides' },
				},
				{
					label: 'Templates',
					autogenerate: { directory: 'templates' },
				},
			],
		}),
	],
});
