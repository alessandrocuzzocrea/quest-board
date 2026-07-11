import { redirect } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export const prerender = false;
export const ssr = false;

export const load: PageLoad = async ({ params, fetch }) => {
	const { slug } = params;
	const res = await fetch(`/api/v1/boards/by-slug/${slug}`);
	if (!res.ok) {
		redirect(302, '/');
	}
	return await res.json();
};
