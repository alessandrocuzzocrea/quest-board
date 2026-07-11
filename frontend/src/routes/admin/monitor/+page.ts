import type { PageLoad } from './$types';

export const ssr = false;

export const load: PageLoad = async ({ fetch }) => {
	const health = await (await fetch('/api/v1/health')).json();
	return { health };
};
