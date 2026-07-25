import type { BoardResponse, FullCard, Board, UserResponse, Label } from './types';

const BASE = '/api/v1';

async function request<T>(method: string, path: string, body?: unknown): Promise<T> {
  const opts: RequestInit = {
    method,
    credentials: 'include',
    headers: {},
  };
  if (body !== undefined) {
    (opts.headers as Record<string, string>)['Content-Type'] = 'application/json';
    opts.body = JSON.stringify(body);
  }
  const res = await fetch(`${BASE}${path}`, opts);
  if (!res.ok) {
    const text = await res.text();
    let msg: string;
    try { msg = JSON.parse(text).error || text; } catch { msg = text; }
    throw new Error(msg);
  }
  return res.json();
}

export const API = {
  login:        (username: string, password: string) => request<UserResponse>('POST', '/auth/login', { username, password }),
  register:     (username: string, password: string, name?: string) => request<UserResponse>('POST', '/auth/register', { username, password, name }),
  logout:       () => request<unknown>('POST', '/auth/logout'),
  me:           () => request<UserResponse>('GET', '/auth/me'),
  changePassword: (current_password: string, new_password: string) => request<unknown>('PUT', '/auth/me/password', { current_password, new_password }),

  listBoards:      () => request<Board[]>('GET', '/boards'),
  getBoard:        (id: string) => request<Board>('GET', `/boards/${id}`),
  getBoardBySlug:  (slug: string) => request<BoardResponse>('GET', `/boards/by-slug/${slug}`),
  createBoard:     (data: { name: string; slug?: string }) => request<Board>('POST', '/boards', data),
  updateBoard:     (id: string, data: { name: string }) => request<Board>('PUT', `/boards/${id}`, data),
  deleteBoard:     (id: string) => request<unknown>('DELETE', `/boards/${id}`),

  createList:  (data: { board_id: string; name: string }) => request<unknown>('POST', '/lists', data),
  updateList:  (id: string, data: { name?: string }) => request<unknown>('PUT', `/lists/${id}`, data),
  deleteList:  (id: string) => request<unknown>('DELETE', `/lists/${id}`),

  createCard:  (data: { list_id: string; name: string; description?: string }) => request<unknown>('POST', '/cards', data),
  getCard:     (id: string) => request<FullCard>('GET', `/cards/${id}`),
  updateCard:  (id: string, data: Record<string, unknown>) => request<unknown>('PUT', `/cards/${id}`, data),
  deleteCard:  (id: string) => request<unknown>('DELETE', `/cards/${id}`),
  moveCard:    (id: string, data: { list_id: string; position: number }) => request<unknown>('PUT', `/cards/${id}/move`, data),

  addCardMember:    (id: string, data: { user_id: string }) => request<unknown>('POST', `/cards/${id}/members`, data),
  removeCardMember: (id: string, data: { user_id: string }) => request<unknown>('DELETE', `/cards/${id}/members`, data),
  addCardLabel:     (id: string, data: { label_id: string }) => request<unknown>('POST', `/cards/${id}/labels`, data),
  removeCardLabel:  (id: string, data: { label_id: string }) => request<unknown>('DELETE', `/cards/${id}/labels`, data),

  addTaskList:    (cardId: string, data: { name: string }) => request<unknown>('POST', `/cards/${cardId}/task-lists`, data),
  updateTaskList: (cardId: string, tlId: string, data: { name?: string; hide_completed?: boolean }) => request<unknown>('PUT', `/cards/${cardId}/task-lists/${tlId}`, data),
  deleteTaskList: (cardId: string, tlId: string) => request<unknown>('DELETE', `/cards/${cardId}/task-lists/${tlId}`),
  createTask:     (cardId: string, tlId: string, data: { name: string }) => request<unknown>('POST', `/cards/${cardId}/task-lists/${tlId}/tasks`, data),
  updateTask:     (cardId: string, tlId: string, taskId: string, data: { name?: string; is_done?: boolean }) => request<unknown>('PUT', `/cards/${cardId}/task-lists/${tlId}/tasks/${taskId}`, data),
  deleteTask:     (cardId: string, tlId: string, taskId: string) => request<unknown>('DELETE', `/cards/${cardId}/task-lists/${tlId}/tasks/${taskId}`),

  search: (q: string) => request<{ cards: unknown[]; boards: { id: string; name: string }[] }>('GET', `/search?q=${encodeURIComponent(q)}`),

  createComment: (data: { card_id: string; text: string }) => request<unknown>('POST', '/comments', data),

  listBoardLabels: (boardId: string) => request<Label[]>('GET', `/labels/board/${boardId}`),
  createLabel:     (data: { board_id: string; name: string; color: string }) => request<unknown>('POST', '/labels', data),
  listApiKeys:  () => request<{ id: string; name: string; prefix: string; last_used_at: string | null }[]>('GET', '/api-keys'),
  createApiKey: (data: { name: string }) => request<{ id: string; name: string; key: string }>('POST', '/api-keys', data),
  deleteApiKey: (id: string) => request<unknown>('DELETE', `/api-keys/${id}`),

  // Favorites
  addFavorite:    (boardId: string) => request<unknown>('POST', '/favorites', { board_id: boardId }),
  removeFavorite: (boardId: string) => request<unknown>('DELETE', `/favorites/${boardId}`),

  // Labels
  updateLabel:  (id: string, data: { name?: string; color?: string }) => request<unknown>('PUT', `/labels/${id}`, data),
  deleteLabel:  (id: string) => request<unknown>('DELETE', `/labels/${id}`),
};
