import type { components } from '$lib/types/bindings';

export type User = components['schemas']['UserResponse'];

export async function api<T = unknown>(path: string, options: RequestInit = {}) {
	const res = await fetch(`/api/v1${path}`, {
		credentials: 'include',
		headers: { 'content-type': 'application/json', ...options.headers },
		...options,
	});

	let data: unknown;
	try {
		data = await res.json();
	} catch {
		data = null;
	}

	if (!res.ok) {
		const message = data && typeof data === 'object' && 'error' in data
			? String((data as Record<string, unknown>).error)
			: `request failed (${res.status})`;
		throw new Error(message);
	}

	return data as T;
}
