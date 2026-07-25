import { useState, useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import { API } from './api'
import NavBar from './NavBar'
import type { Board } from './types'

export default function BoardsList() {
  const [boards, setBoards] = useState<Board[]>([])
  const [search, setSearch] = useState('')
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState('')
  const [showCreate, setShowCreate] = useState(false)
  const [newName, setNewName] = useState('')
  const [newSlug, setNewSlug] = useState('')
  const [deleteTarget, setDeleteTarget] = useState<Board | null>(null)
  const navigate = useNavigate()

  const load = async () => {
    try {
      const list = await API.listBoards()
      setBoards(list)
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Failed to load boards')
    }
    setLoading(false)
  }

  useEffect(() => { load() }, [])

  const filtered = search
    ? boards.filter(b => b.name.toLowerCase().includes(search.toLowerCase()))
    : boards

  const createBoard = async () => {
    if (!newName.trim()) return
    try {
      await API.createBoard({ name: newName.trim(), slug: newSlug.trim() || undefined })
      setShowCreate(false)
      setNewName('')
      setNewSlug('')
      await load()
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Failed to create board')
    }
  }

  const deleteBoard = async () => {
    if (!deleteTarget) return
    try {
      await API.deleteBoard(deleteTarget.id)
      setDeleteTarget(null)
      await load()
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Failed to delete board')
    }
  }

  const openBoard = (board: Board) => {
    const namePart = board.name.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '') || 'board'
    navigate(`/board/${board.slug}/${namePart}`)
  }

  if (loading) return <div className="page" style={{ textAlign: 'center', paddingTop: '40vh' }}><div className="spinner" /></div>

  return (
    <>
      <NavBar />
      <div className="page">
        <div className="flex items-center justify-between mb-16">
          <h1>Boards</h1>
          <button className="btn btn-primary" onClick={() => setShowCreate(true)}>+ New Board</button>
        </div>

        <div className="search-bar mb-16" style={{ maxWidth: 400 }}>
          <span className="icon">&#128269;</span>
          <input
            type="search" placeholder="Filter boards..."
            value={search} onChange={e => setSearch(e.target.value)}
          />
        </div>

        {error && <div className="alert alert-error">{error}</div>}

        {filtered.length === 0 ? (
          <div className="empty-state">
            <div className="icon">{search ? '&#128269;' : '&#128203;'}</div>
            <p>{search ? `No boards match "${search}"` : 'No boards yet. Create one!'}</p>
          </div>
        ) : (
          <div className="board-grid">
            {filtered.map(b => (
              <div key={b.id} className="board-card" onClick={() => openBoard(b)}>
                <div className="actions" onClick={e => e.stopPropagation()}>
                  <button className="btn btn-sm btn-icon" onClick={() => setDeleteTarget(b)} title="Delete">&#128465;</button>
                </div>
                <h3>{b.name}</h3>
                <div className="meta">{b.slug ? `/${b.slug}` : ''}</div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Create board modal */}
      {showCreate && (
        <div className="modal-overlay" onClick={() => setShowCreate(false)}>
          <div className="modal" onClick={e => e.stopPropagation()}>
            <h2>Create Board</h2>
            <div className="form-group">
              <label>Name</label>
              <input type="text" className="input" value={newName} onChange={e => setNewName(e.target.value)} placeholder="My Board" autoFocus />
            </div>
            <div className="form-group">
              <label>Slug (optional)</label>
              <input type="text" className="input" value={newSlug} onChange={e => setNewSlug(e.target.value)} placeholder="my-board" />
            </div>
            <div className="modal-actions">
              <button className="btn" onClick={() => setShowCreate(false)}>Cancel</button>
              <button className="btn btn-primary" onClick={createBoard}>Create</button>
            </div>
          </div>
        </div>
      )}

      {/* Delete confirmation modal */}
      {deleteTarget && (
        <div className="modal-overlay" onClick={() => setDeleteTarget(null)}>
          <div className="modal" onClick={e => e.stopPropagation()}>
            <h2>Delete Board</h2>
            <p>Delete "{deleteTarget.name}"? This cannot be undone.</p>
            <div className="modal-actions">
              <button className="btn" onClick={() => setDeleteTarget(null)}>Cancel</button>
              <button className="btn btn-danger" onClick={deleteBoard}>Delete</button>
            </div>
          </div>
        </div>
      )}
    </>
  )
}
