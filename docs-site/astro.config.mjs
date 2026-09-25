// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

// https://astro.build/config
export default defineConfig({
	site: 'https://dornglut.github.io/runenwerk',
	base: '/runenwerk',
	integrations: [
		starlight({
			title: 'Runenwerk Docs',
			sidebar: [
				{
					label: 'Start here',
					items: [
						{ label: 'Runenwerk Docs', link: '/' },
						{ label: 'Domain', link: '/domain/00-overview/' },
						{ label: 'Engine', link: '/engine/' },
						{
							label: 'Runenwerk Editor',
							link: '/apps/runenwerk-editor/current-architecture/',
						},
						{ label: 'Networking', link: '/net/readme/' },
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
