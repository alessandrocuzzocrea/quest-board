import { useMemo, useRef } from 'react'
import type { ListWithCards } from './types'

interface Props {
  lists: ListWithCards[]
  onSelectCard: (cardId: string) => void
}

interface GanttItem {
  id: string
  name: string
  start: string | null
  end: string | null
  listName: string
  color: string
}

const MONTH_NAMES = ['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec']

export default function GanttChart({ lists, onSelectCard }: Props) {
  const containerRef = useRef<HTMLDivElement>(null)

  const { items, minDate, maxDate, dayWidth } = useMemo(() => {
    const allItems: GanttItem[] = []
    let min = new Date('2099-01-01')
    let max = new Date('1970-01-01')

    lists.forEach(list => {
      ;(list.cards || []).forEach(c => {
        const start = c.start_date || c.created_at || null
        const end = c.due_date || c.updated_at || null
        if (!start && !end) return
        if (start) {
          const d = new Date(start)
          if (d < min) min = d
        }
        if (end) {
          const d = new Date(end)
          if (d > max) max = d
        }
        allItems.push({
          id: c.id,
          name: c.name,
          start: start ? new Date(start).toISOString() : null,
          end: end ? new Date(end).toISOString() : null,
          listName: list.name || '',
          color: c.labels && c.labels.length > 0 ? c.labels[0].color || '#4f8cff' : '#4f8cff',
        })
      })
    })

    if (allItems.length === 0) return { items: allItems, minDate: min, maxDate: max, dayWidth: 40 }

    const pad = 7
    min = new Date(min.getTime() - pad * 86400000)
    max = new Date(max.getTime() + pad * 86400000)
    const days = Math.ceil((max.getTime() - min.getTime()) / 86400000) + 1
    if (days < 7) {
      max = new Date(min.getTime() + 13 * 86400000)
    }

    return { items: allItems, minDate: min, maxDate: max, dayWidth: 40 }
  }, [lists])

  if (items.length === 0) {
    return (
      <div className="empty-state">
        <p>No cards with dates to show on Gantt chart.</p>
      </div>
    )
  }

  const days = Math.ceil((maxDate.getTime() - minDate.getTime()) / 86400000) + 1

  // Build month headers
  const months: { name: string; width: number }[] = []
  const cursor = new Date(minDate.getFullYear(), minDate.getMonth(), 1)
  while (cursor <= maxDate) {
    const monthEnd = new Date(cursor.getFullYear(), cursor.getMonth() + 1, 0)
    const end = monthEnd > maxDate ? maxDate : monthEnd
    const monthDays = Math.round((end.getTime() - Math.max(cursor.getTime(), minDate.getTime())) / 86400000) + 1
    months.push({ name: MONTH_NAMES[cursor.getMonth()], width: monthDays * dayWidth })
    cursor.setMonth(cursor.getMonth() + 1)
  }

  const dateToX = (d: Date) => {
    const ref = new Date(minDate.getTime() - minDate.getTimezoneOffset() * 60000)
    const target = new Date(d.getTime() - d.getTimezoneOffset() * 60000)
    return Math.round((target.getTime() - ref.getTime()) / 86400000) * dayWidth
  }

  return (
    <div className="gantt-chart" ref={containerRef}>
      {/* Header row */}
      <div style={{ display: 'flex', borderBottom: '1px solid var(--border)', background: 'var(--bg2)' }}>
        <div style={{ minWidth: 200, padding: 8, fontWeight: 600, fontSize: 12 }}>Task</div>
        <div style={{ flex: 1, position: 'relative', minWidth: days * dayWidth }}>
          {months.map((m, i) => (
            <div key={i} style={{
              display: 'inline-block',
              textAlign: 'center',
              width: m.width,
              padding: '8px 0',
              borderRight: '1px solid var(--border)',
              fontSize: 12,
              fontWeight: 500,
              color: 'var(--text2)',
            }}>{m.name}</div>
          ))}
        </div>
      </div>

      {/* Rows */}
      {items.map((item) => {
        const sx = item.start ? dateToX(new Date(item.start)) : dateToX(maxDate)
        const ex = item.end ? dateToX(new Date(item.end)) : dateToX(minDate)
        const w = Math.max(dayWidth, ex - sx + dayWidth)

        return (
          <div key={item.id} className="gantt-row" onClick={() => onSelectCard(item.id)}>
            <div className="gantt-label">
              {item.name}
              <span style={{ color: 'var(--text2)', fontSize: 11, marginLeft: 4 }}>{item.listName}</span>
            </div>
            <div className="gantt-bar-wrap">
              <div className="gantt-bar" style={{
                left: sx,
                width: w,
                background: item.color,
              }} />
            </div>
          </div>
        )
      })}
    </div>
  )
}
