import { useState, useEffect, useCallback } from 'react'
import { useParams } from 'react-router-dom'
import { API } from './api'
import NavBar from './NavBar'
import KanbanBoard from './KanbanBoard'
import type { BoardResponse, ListWithCards, Board } from './types'
import GanttChart from './GanttChart'
import CardModal from './CardModal'

export default function BoardPage() {
  const { slug } = useParams<{ slug: string }>()
  const [board, setBoard] = useState<Board | null>(null)
  const [lists, setLists] = useState<ListWithCards[]>([])
  const [view, setView] = useState<'kanban' | 'gantt'>('kanban')
  const [selectedCardId, setSelectedCardId] = useState<string | null>(null)
  const [boardError, setBoardError] = useState('')
  const [editingName, setEditingName] = useState(false)
  const [editNameValue, setEditNameValue] = useState('')

  const loadBoard = useCallback(async () => {
    if (!slug) return
    try {
      const data: BoardResponse = await API.getBoardBySlug(slug)
      setBoard(data.board)
      setLists(data.lists || [])
    } catch (err: unknown) {
      setBoardError(err instanceof Error ? err.message : 'Failed to load board')
    }
  }, [slug])

  useEffect(() => { loadBoard() }, [loadBoard])

  // SSE for real-time updates
  useEffect(() => {
    const es = new EventSource('/api/v1/events')
    let timer: ReturnType<typeof setTimeout> | null = null
    es.addEventListener('change', () => {
      if (timer) clearTimeout(timer)
      timer = setTimeout(loadBoard, 1000)
    })
    es.onerror = () => console.warn('SSE connection lost, reconnecting...')
    return () => { es.close(); if (timer) clearTimeout(timer) }
  }, [loadBoard])

  const updateBoard = async (data: { name: string }) => {
    if (!board) return
    try {
      await API.updateBoard(board.id, data)
      await loadBoard()
    } catch (err: unknown) {
      setBoardError(err instanceof Error ? err.message : 'Failed to update board')
    }
  }

  const renameStart = () => {
    if (!board) return
    setEditNameValue(board.name)
    setEditingName(true)
  }

  const renameSave = async () => {
    if (!board || !editNameValue.trim() || editNameValue.trim() === board.name) {
      setEditingName(false)
      return
    }
    await updateBoard({ name: editNameValue.trim() })
    setEditingName(false)
  }

  const deleteBoard = async () => {
    if (!board || !confirm('Delete this entire board? This cannot be undone.')) return
    try {
      await API.deleteBoard(board.id)
      window.location.href = '/boards'
    } catch (err: unknown) {
      setBoardError(err instanceof Error ? err.message : 'Failed to delete board')
    }
  }

  const addList = async () => {
    if (!board) return
    const name = prompt('List name:')
    if (!name?.trim()) return
    try {
      await API.createList({ board_id: board.id, name: name.trim() })
      await loadBoard()
    } catch (err: unknown) {
      setBoardError(err instanceof Error ? err.message : 'Failed to create list')
    }
  }

  const updateListName = async (listId: string, name: string) => {
    if (!name.trim()) return
    try {
      await API.updateList(listId, { name: name.trim() })
      await loadBoard()
    } catch (err: unknown) {
      setBoardError(err instanceof Error ? err.message : 'Failed to update list')
    }
  }

  const deleteList = async (listId: string) => {
    if (!confirm('Delete this list and all its cards?')) return
    try {
      await API.deleteList(listId)
      await loadBoard()
    } catch (err: unknown) {
      setBoardError(err instanceof Error ? err.message : 'Failed to delete list')
    }
  }

  const createCard = async (listId: string, name: string, description?: string) => {
    try {
      await API.createCard({ list_id: listId, name, description })
      await loadBoard()
    } catch (err: unknown) {
      setBoardError(err instanceof Error ? err.message : 'Failed to create card')
    }
  }

  const moveCard = async (cardId: string, listId: string, position: number) => {
    try {
      await API.moveCard(cardId, { list_id: listId, position })
      await loadBoard()
    } catch (err: unknown) {
      setBoardError(err instanceof Error ? err.message : 'Failed to move card')
    }
  }

  const deleteCardHandler = async (cardId: string) => {
    if (!confirm('Delete this card?')) return
    if (selectedCardId === cardId) setSelectedCardId(null)
    try {
      await API.deleteCard(cardId)
      await loadBoard()
    } catch (err: unknown) {
      setBoardError(err instanceof Error ? err.message : 'Failed to delete card')
    }
  }

  const handleCardUpdated = () => {
    if (selectedCardId) {
      // Force refresh card modal on next open
      setSelectedCardId(null)
    }
    loadBoard()
  }

  return (
    <>
      <NavBar />
      <div className="page-wide">
        {boardError && <div className="alert alert-error">{boardError}</div>}

        <div className="board-header">
          <div className="flex items-center justify-between">
            <div>
              {editingName ? (
                <div className="flex items-center gap-8">
                  <input
                    type="text" className="input"
                    style={{ fontSize: 18, fontWeight: 700, padding: '4px 8px' }}
                    value={editNameValue}
                    onChange={e => setEditNameValue(e.target.value)}
                    onKeyDown={e => { if (e.key === 'Enter') renameSave(); if (e.key === 'Escape') setEditingName(false) }}
                    autoFocus
                  />
                  <button className="btn btn-sm btn-primary" onClick={renameSave}>Save</button>
                  <button className="btn btn-sm" onClick={() => setEditingName(false)}>Cancel</button>
                </div>
              ) : (
                <>
                  <h1 id="board-title" style={{ cursor: 'pointer' }} onClick={renameStart}>{board?.name}</h1>
                  <div className="slug" id="board-slug">{board?.slug ? `/${board.slug}` : ''}</div>
                </>
              )}
            </div>
            <div className="flex items-center gap-8">
              <div className="view-toggle">
                <button className={`view-btn ${view === 'kanban' ? 'active' : ''}`} data-view="kanban" onClick={() => setView('kanban')}>Kanban</button>
                <button className={`view-btn ${view === 'gantt' ? 'active' : ''}`} data-view="gantt" onClick={() => setView('gantt')}>Gantt</button>
              </div>
              <button className="btn btn-sm btn-danger" onClick={deleteBoard}>Delete Board</button>
            </div>
          </div>
        </div>

        {view === 'kanban' ? (
          <KanbanBoard
            lists={lists}
            onAddList={addList}
            onUpdateListName={updateListName}
            onDeleteList={deleteList}
            onCreateCard={createCard}
            onMoveCard={moveCard}
            onDeleteCard={deleteCardHandler}
            onSelectCard={setSelectedCardId}
          />
        ) : (
          <GanttChart lists={lists} onSelectCard={setSelectedCardId} />
        )}
      </div>

      {selectedCardId && (
        <CardModal
          cardId={selectedCardId}
          lists={lists}
          onClose={() => setSelectedCardId(null)}
          onUpdated={handleCardUpdated}
        />
      )}
    </>
  )
}
