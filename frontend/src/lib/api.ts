import createClient from 'openapi-fetch';
import type { paths, components } from '$lib/types/api';

export const client = createClient<paths>({
	baseUrl: '/api/v1',
	credentials: 'include',
	headers: { 'content-type': 'application/json' },
});

export type User = components['schemas']['UserResponse'];

export async function api<T = unknown>(path: string, options: RequestInit = {}) {
	const res = await fetch(`/api/v1${path}`, {
		credentials: 'include',
		headers: { 'content-type': 'application/json', ...options.headers },
		...options,
	});
	const data = await res.json();
	if (!res.ok) throw new Error(data.error ?? 'request failed');
	return data as T;
}
