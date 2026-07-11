import type { UserResponse, Board, ListWithCards, Card, CardWithMembers } from '$lib/types/bindings';

export type User = UserResponse;

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

export type { Board, ListWithCards, Card, CardWithMembers, UserResponse };
