import { useState, useMemo, useCallback } from 'react'
import {
  DndContext,
  DragOverlay,
  closestCorners,
  useSensors,
  useSensor,
  PointerSensor,
  type DragStartEvent,
  type DragEndEvent,
  type DragOverEvent,
} from '@dnd-kit/core'
import { SortableContext, verticalListSortingStrategy, useSortable } from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import type { ListWithCards, CardWithMembers } from './types'

interface Props {
  lists: ListWithCards[]
  onAddList: () => void
  onUpdateListName: (listId: string, name: string) => void
  onDeleteList: (listId: string) => void
  onCreateCard: (listId: string, name: string, description?: string) => void
  onMoveCard: (cardId: string, toListId: string, position: number) => void
  onDeleteCard: (cardId: string) => void
  onSelectCard: (cardId: string) => void
}

// Compute position between two cards (or at start/end)
function computePosition(prevPos: number | null, nextPos: number | null): number {
  if (prevPos === null && nextPos === null) return 1000
  if (prevPos === null) return (nextPos as number) / 2
  if (nextPos === null) return prevPos + 1000
  return (prevPos + nextPos) / 2
}

function SortableCard({ card, onSelect, onDelete }: { card: CardWithMembers; onSelect: (id: string) => void; onDelete: (id: string) => void }) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id: card.id,
    data: { type: 'card', card, listId: card.list_id },
  })

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : undefined,
  }

  const today = new Date().toISOString().slice(0, 10)

  return (
    <div ref={setNodeRef} style={style} {...attributes} {...listeners} className="kanban-card" onClick={() => onSelect(card.id)}>
      <h4>{card.name}</h4>
      {card.description && <div className="desc">{card.description}</div>}
      {card.labels && card.labels.length > 0 && (
        <div className="labels">
          {card.labels.map(l => (
            <span key={l.id} className="label-dot" style={{ background: l.color || '#4f8cff' }} title={l.name} />
          ))}
        </div>
      )}
      <div className="meta-row">
        {card.start_date && <span className="date-badge date-start">&#128197; {card.start_date}</span>}
        {card.due_date && (
          <span className={`date-badge ${card.due_date.match(/^\d{4}-\d{2}-\d{2}$/) && card.due_date < today && !card.is_due_completed ? 'date-overdue' : 'date-due'}`}>
            &#128197; {card.due_date}{card.is_due_completed ? ' \u2713' : ''}
          </span>
        )}
        {card.comments_count > 0 && <span>&#128172; {card.comments_count}</span>}
      </div>
      <button
        className="btn btn-sm btn-icon"
        style={{ position: 'absolute', top: 4, right: 4, opacity: 0.3 }}
        onClick={e => { e.stopPropagation(); onDelete(card.id) }}
        title="Delete card"
      >&times;</button>
    </div>
  )
}

function DroppableColumn({ list, cards, onUpdateListName, onDeleteList, onCreateCard, onSelectCard, onDeleteCard }: {
  list: ListWithCards
  cards: CardWithMembers[]
  onUpdateListName: (id: string, name: string) => void
  onDeleteList: (id: string) => void
  onCreateCard: (listId: string, name: string, desc?: string) => void
  onSelectCard: (id: string) => void
  onDeleteCard: (id: string) => void
}) {
  const [editingName, setEditingName] = useState(false)
  const [nameValue, setNameValue] = useState(list.name || '')
  const [adding, setAdding] = useState(false)
  const [newName, setNewName] = useState('')
  const [newDesc, setNewDesc] = useState('')

  const cardIds = useMemo(() => cards.map(c => c.id), [cards])

  const saveName = () => {
    if (nameValue.trim() && nameValue.trim() !== (list.name || '')) {
      onUpdateListName(list.id, nameValue.trim())
    }
    setEditingName(false)
  }

  const addCard = () => {
    if (!newName.trim()) return
    onCreateCard(list.id, newName.trim(), newDesc.trim() || undefined)
    setNewName('')
    setNewDesc('')
    setAdding(false)
  }

  return (
    <div className="column">
      <div className="column-header">
        {editingName ? (
          <input
            type="text" className="input"
            style={{ fontSize: 13, padding: '2px 6px' }}
            value={nameValue}
            onChange={e => setNameValue(e.target.value)}
            onBlur={saveName}
            onKeyDown={e => { if (e.key === 'Enter') saveName(); if (e.key === 'Escape') setEditingName(false) }}
            autoFocus
          />
        ) : (
          <span onClick={() => { setNameValue(list.name || ''); setEditingName(true) }}>
            {list.name} <span className="count">({cards.length})</span>
          </span>
        )}
        <button className="btn btn-sm btn-icon" onClick={() => onDeleteList(list.id)} title="Delete list">&times;</button>
      </div>
      <div className="column-body">
        <SortableContext items={cardIds} strategy={verticalListSortingStrategy}>
          {cards.map(card => (
            <SortableCard key={card.id} card={card} onSelect={onSelectCard} onDelete={onDeleteCard} />
          ))}
        </SortableContext>
        {adding && (
          <div style={{ background: 'var(--surface)', border: '1px solid var(--border)', borderRadius: 6, padding: 8, marginBottom: 8 }}>
            <input type="text" className="input" style={{ marginBottom: 4, fontSize: 13 }}
              value={newName} onChange={e => setNewName(e.target.value)}
              placeholder="Card name" autoFocus
              onKeyDown={e => { if (e.key === 'Enter') addCard(); if (e.key === 'Escape') setAdding(false) }}
            />
            <input type="text" className="input" style={{ fontSize: 12 }}
              value={newDesc} onChange={e => setNewDesc(e.target.value)}
              placeholder="Description (optional)"
            />
            <div className="flex gap-8" style={{ marginTop: 4 }}>
              <button className="btn btn-sm btn-primary" onClick={addCard}>Add</button>
              <button className="btn btn-sm" onClick={() => setAdding(false)}>Cancel</button>
            </div>
          </div>
        )}
      </div>
      <div className="column-footer">
        <button className="btn btn-sm" style={{ width: '100%' }} onClick={() => setAdding(true)}>+ Add Card</button>
      </div>
    </div>
  )
}

export default function KanbanBoard({ lists, onAddList, onUpdateListName, onDeleteList, onCreateCard, onMoveCard, onDeleteCard, onSelectCard }: Props) {
  const [activeCard, setActiveCard] = useState<CardWithMembers | null>(null)

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 5 } })
  )

  const handleDragStart = useCallback((event: DragStartEvent) => {
    const { active } = event
    if (active.data.current?.type === 'card') {
      setActiveCard(active.data.current.card as CardWithMembers)
    }
  }, [])

  const handleDragOver = useCallback((event: DragOverEvent) => {
    const { over } = event
    if (!over || !activeCard) return
  }, [activeCard])

  const handleDragEnd = useCallback((event: DragEndEvent) => {
    const { active, over } = event
  setActiveCard(null)
    if (!over || !active.data.current) return

    const cardData = active.data.current.card as CardWithMembers
    const fromListId = cardData.list_id

    // Determine target list
    const targetListId = over.data.current?.type === 'card'
      ? (over.data.current.card as CardWithMembers).list_id
      : over.id as string

    const targetList = lists.find(l => l.id === targetListId)
    if (!targetList) return

    const cardsInTarget = targetList.cards || []
    
    // Calculate position
    let position: number
    const overIndex = cardsInTarget.findIndex(c => c.id === over.id)
    if (overIndex === -1) {
      // Dropped on the column itself
      position = cardsInTarget.length > 0
        ? cardsInTarget[cardsInTarget.length - 1].position + 1000
        : 1000
    } else {
      const overCard = cardsInTarget[overIndex]
      if (overCard.id === cardData.id) {
        // Same position - no move needed within the same list
        if (fromListId === targetListId) return
      }
      const prevCard = overIndex > 0 ? cardsInTarget[overIndex - 1] : null
      const nextCard = cardsInTarget[overIndex]
      position = computePosition(
        prevCard && prevCard.id !== cardData.id ? prevCard.position : null,
        nextCard ? nextCard.position : null
      )
    }

    // If dropping on the same list, check if position actually changed
    if (fromListId === targetListId) {
      const currentCards = lists.find(l => l.id === fromListId)?.cards || []
      const currentCard = currentCards.find(c => c.id === cardData.id)
      if (currentCard) {
        // Only move if position changes meaningfully
        const sortedCards = [...currentCards].sort((a, b) => a.position - b.position)
        const currentIdx = sortedCards.findIndex(c => c.id === cardData.id)
        const overIdx = sortedCards.findIndex(c => c.id === over.id)
        if (currentIdx === overIdx) return
      }
    }

    onMoveCard(cardData.id, targetListId, position)
  }, [lists, onMoveCard])

  if (lists.length === 0) {
    return (
      <div className="kanban">
        <div className="empty-state" style={{ margin: '0 auto' }}>
          <div className="icon">&#128196;</div>
          <p>No lists. Add one!</p>
        </div>
        <button className="btn btn-primary" style={{ alignSelf: 'flex-start', marginTop: 16 }} onClick={onAddList}>+ Add List</button>
      </div>
    )
  }

  return (
    <DndContext
      sensors={sensors}
      collisionDetection={closestCorners}
      onDragStart={handleDragStart}
      onDragOver={handleDragOver}
      onDragEnd={handleDragEnd}
    >
      <div className="kanban">
        {lists.map(list => (
          <DroppableColumn
            key={list.id}
            list={list}
            cards={list.cards || []}
            onUpdateListName={onUpdateListName}
            onDeleteList={onDeleteList}
            onCreateCard={onCreateCard}
            onSelectCard={onSelectCard}
            onDeleteCard={onDeleteCard}
          />
        ))}
        <div
          className="column"
          style={{ minWidth: 200, border: '2px dashed var(--border)', background: 'transparent', display: 'flex', alignItems: 'center', justifyContent: 'center', cursor: 'pointer' }}
          onClick={onAddList}
        >
          <span style={{ color: 'var(--text2)' }}>+ Add List</span>
        </div>
      </div>

      <DragOverlay>
        {activeCard && (
          <div className="kanban-card" style={{ boxShadow: '0 8px 24px rgba(0,0,0,.15)', maxWidth: 280 }}>
            <h4>{activeCard.name}</h4>
          </div>
        )}
      </DragOverlay>
    </DndContext>
  )
}
