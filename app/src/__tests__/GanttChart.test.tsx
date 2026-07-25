import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import GanttChart from '../GanttChart'
import type { ListWithCards } from '../types'

const mockList: ListWithCards = {
  id: 'l1',
  board_id: 'b1',
  name: 'To Do',
  position: 0,
  type: 'default',
  color: null,
  cards: [
    {
      id: 'c1',
      board_id: 'b1',
      list_id: 'l1',
      position: 0,
      name: 'Task A',
      description: null,
      start_date: '2026-07-20',
      due_date: '2026-07-25',
      is_due_completed: false,
      is_closed: false,
      created_by: 'u1',
      members: [],
      labels: [{ id: 'lab1', board_id: 'b1', name: 'bug', color: '#e74c3c', position: 0, created_at: '', updated_at: '' }],
      comments_count: 0n,
      checklists: [],
      created_at: '',
      updated_at: '',
    },
    {
      id: 'c2',
      board_id: 'b1',
      list_id: 'l1',
      position: 1,
      name: 'Task B',
      description: 'no dates',
      start_date: null,
      due_date: null,
      is_due_completed: false,
      is_closed: false,
      created_by: 'u1',
      members: [],
      labels: [],
      comments_count: 0n,
      checklists: [],
      created_at: '',
      updated_at: '',
    },
  ],
  created_at: '',
  updated_at: '',
}

describe('GanttChart', () => {
  const onSelectCard = vi.fn()

  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('shows empty state when no lists have dated cards', () => {
    // Card without any dates — start/end remain null even with fallback
    const cardNoDates = { ...mockList.cards[1] } // already has no dates and no created_at
    const emptyList: ListWithCards[] = [{
      ...mockList,
      cards: [{ ...cardNoDates, created_at: '', updated_at: '' }],
    }]
    render(<GanttChart lists={emptyList} onSelectCard={onSelectCard} />)
    expect(screen.getByText('No cards with dates to show on Gantt chart.')).toBeInTheDocument()
  })

  it('renders rows for cards with dates', () => {
    render(<GanttChart lists={[mockList]} onSelectCard={onSelectCard} />)
    expect(screen.getByText('Task A')).toBeInTheDocument()
    // "To Do" appears once per card — use getAllByText
    const labels = screen.getAllByText('To Do')
    expect(labels.length).toBeGreaterThanOrEqual(1)
  })

  it('renders the month header', () => {
    render(<GanttChart lists={[mockList]} onSelectCard={onSelectCard} />)
    expect(screen.getByText('Jul')).toBeInTheDocument()
  })

  it('calls onSelectCard when a row is clicked', async () => {
    const user = userEvent.setup()
    render(<GanttChart lists={[mockList]} onSelectCard={onSelectCard} />)
    await user.click(screen.getByText('Task A'))
    expect(onSelectCard).toHaveBeenCalledWith('c1')
  })

  it('renders multiple lists with cards', () => {
    const list2: ListWithCards = {
      ...mockList,
      id: 'l2',
      name: 'Done',
      cards: [{
        ...mockList.cards[0],
        id: 'c3',
        name: 'Task C',
        start_date: '2026-07-01',
        due_date: '2026-07-10',
      }],
    }
    render(<GanttChart lists={[mockList, list2]} onSelectCard={onSelectCard} />)
    expect(screen.getByText('Task A')).toBeInTheDocument()
    expect(screen.getByText('Task C')).toBeInTheDocument()
  })
})
