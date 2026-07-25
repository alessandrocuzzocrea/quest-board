import { useState, useEffect, useCallback } from 'react'
import { API } from './api'
import type { FullCard, ListWithCards, Label } from './types'

interface Props {
  cardId: string
  lists: ListWithCards[]
  onClose: () => void
  onUpdated: () => void
}

export default function CardModal({ cardId, lists, onClose, onUpdated }: Props) {
  const [card, setCard] = useState<FullCard | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState('')
  const [editDesc, setEditDesc] = useState(false)
  const [descValue, setDescValue] = useState('')
  const [editDue, setEditDue] = useState(false)
  const [dueValue, setDueValue] = useState('')
  const [editStart, setEditStart] = useState(false)
  const [startValue, setStartValue] = useState('')
  const [tab, setTab] = useState<'comments' | 'actions'>('comments')
  const [commentText, setCommentText] = useState('')
  const [allLabels, setAllLabels] = useState<Label[]>([])
  const [showLabelPicker, setShowLabelPicker] = useState(false)
  // Task list add
  const [newTlName, setNewTlName] = useState('')
  const [addingTl, setAddingTl] = useState(false)

  const loadCard = useCallback(async () => {
    try {
      const c = await API.getCard(cardId)
      setCard(c)
      setDescValue(c.description || '')
      setDueValue(c.due_date || '')
      setStartValue(c.start_date || '')
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Failed to load card')
    }
    setLoading(false)
  }, [cardId])

  useEffect(() => { loadCard() }, [loadCard])

  const saveField = async (field: string, value: unknown) => {
    if (!card) return
    try {
      await API.updateCard(card.id, { [field]: value })
      await loadCard()
      onUpdated()
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Failed to update')
    }
  }

  const addComment = async () => {
    if (!card || !commentText.trim()) return
    try {
      await API.createComment({ card_id: card.id, text: commentText.trim() })
      setCommentText('')
      await loadCard()
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Failed to add comment')
    }
  }

  const toggleLabel = async (label: Label) => {
    if (!card) return
    const has = (card.labels || []).find(l => l.id === label.id)
    try {
      if (has) {
        await API.removeCardLabel(card.id, { label_id: label.id })
      } else {
        await API.addCardLabel(card.id, { label_id: label.id })
      }
      await loadCard()
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Failed to update label')
    }
  }

  const loadLabelPicker = async () => {
    if (!card) return
    setShowLabelPicker(!showLabelPicker)
    if (!showLabelPicker) {
      try {
        const labels = await API.listBoardLabels(card.board_id)
        setAllLabels(labels)
      } catch (err: unknown) {
        setError(err instanceof Error ? err.message : 'Failed to load labels')
      }
    }
  }

  const addTaskList = async () => {
    if (!card || !newTlName.trim()) return
    try {
      await API.addTaskList(card.id, { name: newTlName.trim() })
      setNewTlName('')
      setAddingTl(false)
      await loadCard()
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Failed to add task list')
    }
  }

  const toggleTask = async (tlId: string, taskId: string, isDone: boolean) => {
    if (!card) return
    try {
      await API.updateTask(card.id, tlId, taskId, { is_done: isDone })
      await loadCard()
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Failed to update task')
    }
  }

  const addTask = async (tlId: string) => {
    if (!card) return
    const name = prompt('Task name:')
    if (!name?.trim()) return
    try {
      await API.createTask(card.id, tlId, { name: name.trim() })
      await loadCard()
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Failed to add task')
    }
  }

  const deleteTask = async (tlId: string, taskId: string) => {
    if (!card) return
    try {
      await API.deleteTask(card.id, tlId, taskId)
      await loadCard()
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Failed to delete task')
    }
  }

  // Escape to close
  useEffect(() => {
    const handler = (e: KeyboardEvent) => { if (e.key === 'Escape') onClose() }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [onClose])

  if (loading) return null

  const list = card ? lists.find(l => l.id === card.list_id) : null
  const checklists = card?.checklists || []

  return (
    <div className="modal-overlay" onClick={e => { if (e.target === e.currentTarget) onClose() }}>
      <div className="modal modal-card-v2" onClick={e => e.stopPropagation()}>
        {error && <div className="alert alert-error" style={{ margin: '0 24px' }}>{error}</div>}

        <div className="card-modal-header">
          <div>
            <h2 id="panel-title">{card?.name}</h2>
            <div className="card-list-label" id="panel-list-name">in {list?.name || 'Unknown list'}</div>
          </div>
          <button className="btn btn-sm btn-icon" onClick={onClose}>&times;</button>
        </div>

        <div className="card-modal-body">
          {/* Main column */}
          <div className="card-modal-main">
            {/* Description */}
            <div className="panel-section">
              <div className="section-header">
                <span className="section-icon">&#128221;</span>
                <h3>Description</h3>
                <button className="btn btn-sm section-edit-btn" onClick={() => setEditDesc(!editDesc)}>
                  {editDesc ? 'Save' : 'Edit'}
                </button>
              </div>
              {editDesc ? (
                <div>
                  <textarea
                    className="input"
                    value={descValue}
                    onChange={e => setDescValue(e.target.value)}
                    autoFocus
                  />
                  <div className="flex gap-8 mt-8">
                    <button className="btn btn-sm btn-primary" onClick={async () => { await saveField('description', descValue || null); setEditDesc(false) }}>Save</button>
                    <button className="btn btn-sm" onClick={() => { setDescValue(card?.description || ''); setEditDesc(false) }}>Cancel</button>
                  </div>
                </div>
              ) : (
                <p style={{ color: 'var(--text2)', fontSize: 13 }} className={card?.description ? '' : 'desc-placeholder'}>
                  {card?.description || 'No description'}
                </p>
              )}
            </div>

            {/* Checklist */}
            <div className="panel-section">
              <div className="section-header">
                <span className="section-icon">&#9744;</span>
                <h3>Checklist</h3>
                <button className="btn btn-sm section-edit-btn" onClick={() => setAddingTl(!addingTl)}>+</button>
              </div>
              {addingTl && (
                <div className="flex gap-8 mb-8">
                  <input type="text" className="input" style={{ flex: 1, fontSize: 13 }}
                    value={newTlName} onChange={e => setNewTlName(e.target.value)}
                    placeholder="List name" autoFocus
                    onKeyDown={e => { if (e.key === 'Enter') addTaskList(); if (e.key === 'Escape') setAddingTl(false) }}
                  />
                  <button className="btn btn-sm btn-primary" onClick={addTaskList}>Add</button>
                </div>
              )}
              {checklists.map(tl => {
                const done = tl.tasks.filter(t => t.is_done).length
                const total = tl.tasks.length
                return (
                  <div key={tl.id} className="task-list-block">
                    <div className="task-list-header">
                      <strong>{tl.name}</strong>
                      {total > 0 && <span className="task-progress-text">{done}/{total}</span>}
                      <button className="btn btn-sm btn-icon" style={{ marginLeft: 'auto' }}
                        onClick={async () => { await API.deleteTaskList(card!.id, tl.id); await loadCard() }}
                      >&times;</button>
                    </div>
                    {total > 0 && (
                      <div className="task-progress-bar mb-8">
                        <div className="task-progress-fill" style={{ width: `${(done / total) * 100}%` }} />
                      </div>
                    )}
                    {tl.tasks.map(t => (
                      <div key={t.id} className="checklist-item">
                        <input type="checkbox" checked={t.is_done} onChange={e => toggleTask(tl.id, t.id, e.target.checked)} />
                        <span className={t.is_done ? 'task-done' : ''}>{t.name}</span>
                        <button className="btn btn-sm btn-icon task-del-btn" onClick={() => deleteTask(tl.id, t.id)}>&times;</button>
                      </div>
                    ))}
                    <div className="task-add-row">
                      <button className="btn btn-sm" style={{ fontSize: 12 }} onClick={() => addTask(tl.id)}>+ Add task</button>
                    </div>
                  </div>
                )
              })}
            </div>

            {/* Comments / Actions tabs */}
            <div className="panel-section">
              <div className="comment-tabs">
                <div className={`tab ${tab === 'comments' ? 'active' : ''}`} onClick={() => setTab('comments')}>
                  Comments {card?.comments?.length ? `(${card.comments.length})` : ''}
                </div>
                <div className={`tab ${tab === 'actions' ? 'active' : ''}`} onClick={() => setTab('actions')}>
                  Activity {card?.actions?.length ? `(${card.actions.length})` : ''}
                </div>
              </div>

              <div className={`tab-panel ${tab === 'comments' ? 'active' : ''}`}>
                <div className="comments-list">
                  {card?.comments && card.comments.length > 0 ? card.comments.map(c => (
                    <div key={c.id} className="comment">
                      <div className="comment-header">
                        <strong>{c.user?.username || 'User'}</strong>
                        <span className="text-sm" style={{ color: 'var(--text2)' }}>{c.created_at}</span>
                      </div>
                      <div className="text">{c.text}</div>
                    </div>
                  )) : <p style={{ color: 'var(--text2)', fontSize: 13 }}>No comments yet.</p>}
                </div>
                <div className="flex gap-8" style={{ marginTop: 8 }}>
                  <input type="text" className="input" style={{ flex: 1, fontSize: 13 }}
                    value={commentText} onChange={e => setCommentText(e.target.value)}
                    placeholder="Add a comment..."
                    onKeyDown={e => { if (e.key === 'Enter') addComment() }}
                  />
                  <button className="btn btn-sm btn-primary" onClick={addComment}>Send</button>
                </div>
              </div>

              <div className={`tab-panel ${tab === 'actions' ? 'active' : ''}`}>
                {(card?.actions || []).length > 0 ? card!.actions.map(a => (
                  <div key={a.id} className="comment">
                    <div className="comment-header">
                      <strong>{/* action has user_id not user_name, skipping name */}</strong>
                      <span className="text-sm" style={{ color: 'var(--text2)' }}>{a.type}</span>
                      <span style={{ color: 'var(--text2)', fontSize: 11, marginLeft: 8 }}>{a.created_at}</span>
                    </div>
                  </div>
                )) : <p style={{ color: 'var(--text2)', fontSize: 13 }}>No activity yet.</p>}
              </div>
            </div>
          </div>

          {/* Sidebar */}
          <div className="card-modal-sidebar">
            {/* Dates */}
            <div className="panel-section">
              <h3>Dates</h3>
              <div className="sidebar-btn-group">
                <div className="sidebar-btn" onClick={() => setEditStart(!editStart)}>
                  <span className="sidebar-btn-icon">&#128197;</span>
                  {editStart ? (
                    <input type="date" className="input" style={{ fontSize: 12, padding: '2px 4px' }}
                      value={startValue} onChange={e => setStartValue(e.target.value)}
                      onBlur={() => saveField('start_date', startValue || null)}
                    />
                  ) : (
                    <span>{card?.start_date || 'Start date'}</span>
                  )}
                </div>
                <div className="sidebar-btn" onClick={() => setEditDue(!editDue)}>
                  <span className="sidebar-btn-icon">&#128197;</span>
                  {editDue ? (
                    <input type="date" className="input" style={{ fontSize: 12, padding: '2px 4px' }}
                      value={dueValue} onChange={e => setDueValue(e.target.value)}
                      onBlur={() => saveField('due_date', dueValue || null)}
                    />
                  ) : (
                    <span>{card?.due_date || 'Due date'}</span>
                  )}
                </div>
              </div>
            </div>

            {/* Labels */}
            <div className="panel-section">
              <h3>Labels</h3>
              <div id="panel-labels">
                {(card?.labels || []).length > 0 ? card!.labels.map(l => (
                  <span key={l.id} className="label-badge" style={{ background: `${l.color}20`, border: `1px solid ${l.color}`, color: l.color, marginRight: 4, marginBottom: 4, display: 'inline-block' }}>
                    {l.name}
                  </span>
                )) : <span style={{ color: 'var(--text2)', fontSize: 13 }}>None</span>}
              </div>
              <button className="btn btn-sm" style={{ marginTop: 4, width: '100%' }} onClick={loadLabelPicker}>
                {showLabelPicker ? 'Done' : 'Edit Labels'}
              </button>
              {showLabelPicker && (
                <div id="labels-picker" style={{ marginTop: 4 }}>
                  {allLabels.map(l => {
                    const has = (card?.labels || []).find(cl => cl.id === l.id)
                    return (
                      <div key={l.id} style={{ display: 'flex', alignItems: 'center', gap: 6, padding: '4px 0', cursor: 'pointer' }}
                        onClick={() => toggleLabel(l)}>
                        <span className="label-dot" style={{ background: l.color || '#4f8cff' }} />
                        <span style={{ flex: 1, fontSize: 13 }}>{l.name}</span>
                        <span>{has ? '\u2713' : '+'}</span>
                      </div>
                    )
                  })}
                </div>
              )}
            </div>

            {/* Members */}
            <div className="panel-section">
              <h3>Members</h3>
              {(card?.members || []).length > 0 ? card!.members.map(m => (
                <div key={m.id} style={{ display: 'flex', alignItems: 'center', gap: 4, padding: '2px 0' }}>
                  <span style={{ fontSize: 13 }}>{m.username}</span>
                  <button className="btn btn-sm btn-icon" style={{ marginLeft: 'auto' }}
                    onClick={async () => {
                      await API.removeCardMember(card!.id, { user_id: m.id })
                      await loadCard()
                    }}
                  >&times;</button>
                </div>
              )) : <span style={{ color: 'var(--text2)', fontSize: 13 }}>None</span>}
            </div>

            {/* Attachments */}
            <div className="panel-section">
              <h3>Attachments</h3>
              {(card?.attachments || []).length > 0 ? card!.attachments.map(a => (
                <div key={a.id} style={{ display: 'flex', alignItems: 'center', gap: 4, padding: '2px 0' }}>
                  <a href={a.file_path ? `/${a.file_path}` : a.link_url || '#'} target="_blank" style={{ flex: 1, fontSize: 13 }}>{a.name}</a>
                  <span className="text-sm">{a.type === 'file' && a.size ? `${Math.round(Number(a.size) / 1024)}KB` : a.type}</span>
                </div>
              )) : <span style={{ color: 'var(--text2)', fontSize: 13 }}>None</span>}
            </div>

            {/* Actions */}
            <div className="panel-section">
              <div className="sidebar-btn-group">
                <button className="sidebar-btn" onClick={async () => { if (card) { await saveField('is_closed', true) } }}>
                  <span className="sidebar-btn-icon">&#128465;</span>
                  Archive
                </button>
                <button className="sidebar-btn" onClick={async () => {
                  if (card) {
                    await API.deleteCard(card.id)
                    onUpdated()
                    onClose()
                  }
                }}>
                  <span className="sidebar-btn-icon">&#128465;</span>
                  Delete
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
