const BASE = '/api/v1';

export interface User {
	id: string;
	email: string;
	name: string;
	role: string;
}

export async function api(path: string, options: RequestInit = {}) {
	const res = await fetch(`${BASE}${path}`, {
		credentials: 'include',
		headers: { 'content-type': 'application/json', ...options.headers },
		...options,
	});
	const data = await res.json();
	if (!res.ok) throw new Error(data.error ?? 'request failed');
	return data;
}
