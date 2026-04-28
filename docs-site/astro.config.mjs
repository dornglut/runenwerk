// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

// https://astro.build/config
export default defineConfig({
	integrations: [
		starlight({
			title: 'Runenwerk Docs',
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
					label: 'Apps',
					autogenerate: { directory: 'apps' },
				},
				{
					label: 'Adapters',
					autogenerate: { directory: 'adapters' },
				},
				{
					label: 'ADRs',
					autogenerate: { directory: 'adr' },
				},
				{
					label: 'Design',
					autogenerate: { directory: 'design' },
				},
				{
					label: 'Multiplayer',
					autogenerate: { directory: 'multiplayer' },
				},
				{
					label: 'Guidelines',
					autogenerate: { directory: 'guidelines' },
				},
				{
					label: 'Templates',
					autogenerate: { directory: 'templates' },
				},
			],
		}),
	],
});
