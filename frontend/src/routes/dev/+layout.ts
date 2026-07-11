import { dev } from '$app/environment';
import { redirect } from '@sveltejs/kit';

if (!dev) redirect(307, '/');
