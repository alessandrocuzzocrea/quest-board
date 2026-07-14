// API client — wraps fetch with cookie-based auth, error handling
const API = (() => {
  const BASE = '/api/v1';

  async function request(method, path, body) {
    const opts = {
      method,
      headers: { 'Content-Type': 'application/json' },
      credentials: 'same-origin',
    };
    if (body !== undefined) opts.body = JSON.stringify(body);
    const res = await fetch(`${BASE}${path}`, opts);
    if (res.status === 401) {
      if (!window.location.pathname.includes('/login')) {
        window.location.href = '/login';
      }
      throw new Error('Unauthorized');
    }
    const data = await res.json();
    if (!res.ok) throw new Error(data.error || res.statusText);
    return data;
  }

  return {
    get:    (p) => request('GET', p),
    post:   (p, b) => request('POST', p, b),
    put:    (p, b) => request('PUT', p, b),
    del:    (p) => request('DELETE', p),

    // Auth
    login:     (username, password)      => request('POST', '/auth/login', { username, password }),
    register:  (username, password, name) => request('POST', '/auth/register', { username, password, name }),
    logout:    ()                     => request('POST', '/auth/logout'),
    me:        ()                     => request('GET', '/auth/me'),
    updateName: (name)                => request('PUT', '/auth/me', { name }),
    changePassword: (current, newpw) => request('PUT', '/auth/me/password', { current_password: current, new_password: newpw }),

    // Boards
    listBoards:  ()          => request('GET', '/boards'),
    getBoard:    (id)        => request('GET', `/boards/${id}`),
    createBoard: (data)      => request('POST', '/boards', data),
    updateBoard: (id, data)  => request('PUT', `/boards/${id}`, data),
    deleteBoard: (id)        => request('DELETE', `/boards/${id}`),

    // Lists
    createList: (data) => request('POST', '/lists', data),

    // Cards
    createCard:   (data)       => request('POST', '/cards', data),
    getCard:      (id)         => request('GET', `/cards/${id}`),
    updateCard:   (id, data)   => request('PUT', `/cards/${id}`, data),
    deleteCard:   (id)         => request('DELETE', `/cards/${id}`),
    moveCard:     (id, data)   => request('PUT', `/cards/${id}/move`, data),
    addCardMember:  (id, data) => request('POST', `/cards/${id}/members`, data),
    removeCardMember: (id, data) => request('DELETE', `/cards/${id}/members`, data),
    addCardLabel:  (id, data)  => request('POST', `/cards/${id}/labels`, data),
    removeCardLabel: (id, data) => request('DELETE', `/cards/${id}/labels`, data),
    addTaskList:   (id, data)  => request('POST', `/cards/${id}/task-lists`, data),

    // Comments
    createComment: (data) => request('POST', '/comments', data),

    // Labels
    createLabel: (data) => request('POST', '/labels', data),

    // Search
    search: (q) => request('GET', `/search?q=${encodeURIComponent(q)}`),

    // API Keys
    listApiKeys:   ()        => request('GET', '/api-keys'),
    createApiKey:  (data)    => request('POST', '/api-keys', data),
    deleteApiKey:  (id)      => request('DELETE', `/api-keys/${id}`),

    // Health
    health: () => request('GET', '/health'),
  };
})();

// DOM helpers
function $(sel, ctx) { return (ctx || document).querySelector(sel); }
function $$(sel, ctx) { return Array.from((ctx || document).querySelectorAll(sel)); }
function el(tag, attrs, ...kids) {
  const e = document.createElement(tag);
  if (attrs) for (const [k, v] of Object.entries(attrs)) {
    if (k === 'className') e.className = v;
    else if (k.startsWith('on')) e.addEventListener(k.slice(2), v);
    else if (k === 'dataset') Object.assign(e.dataset, v);
    else e.setAttribute(k, v);
  }
  for (const kid of kids) {
    if (kid == null) continue;
    e.append(typeof kid === 'string' ? document.createTextNode(kid) : kid);
  }
  return e;
}

function showAlert(msg, type = 'error') {
  const existing = $('.alert');
  if (existing) existing.remove();
  const a = el('div', { className: `alert alert-${type}` }, msg);
  const target = $('.auth-box') || $('.page') || document.body;
  target.prepend(a);
  setTimeout(() => a.remove(), 5000);
}

async function checkAuth() {
  try {
    const data = await API.me();
    return data.user;
  } catch { return null; }
}

async function requireAuth() {
  const user = await checkAuth();
  if (!user) { window.location.href = '/login'; return null; }
  return user;
}

function navBar(currentPage) {
  const pages = [
    { href: '/boards', label: 'Boards' },
    { href: '/settings', label: 'Settings' },
  ];
  const nav = el('div', { className: 'topbar' },
    el('a', { href: '/boards', className: 'logo' }, 'quest-board'),
    el('nav', {},
      ...pages.map(p => el('a', { href: p.href, style: p.href === currentPage ? 'color: var(--accent)' : '' }, p.label)),
      el('span', { className: 'user-name', id: 'user-name' }),
      el('button', { className: 'btn btn-sm', onClick: async () => { await API.logout(); window.location.href = '/login'; } }, 'Logout'),
    ),
  );
  return nav;
}
