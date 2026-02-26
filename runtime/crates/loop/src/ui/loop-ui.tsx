import React, { useEffect, useMemo, useState } from 'react'
import { Box, Text, useApp, useInput } from 'ink'
import path from 'path'
import { promises as fs } from 'fs'
import { useScreenSize, withFullScreen } from './fullscreen'

type TaskStatus = 'waiting' | 'running' | 'done' | 'failed'

type LoopTask = {
  id: string
  category: string
  description: string
  steps: string[]
  status: TaskStatus
}

type LoopState = {
  meta?: {
    coding_tool?: string
    notes?: string
  }
  tasks: LoopTask[]
}

const LOOP_JSON = path.join(process.cwd(), '.deka', 'loop.json')
const PROGRESS_MD = path.join(process.cwd(), '.deka', 'progress.md')
const REFRESH_MS = 2500
const STATUS_ORDER: TaskStatus[] = ['waiting', 'running', 'done', 'failed']
const OUTER_PADDING = 1

const parseArgs = (argv: string[]) => {
  let iterations: number | undefined
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i]
    if (arg === '--iterations' && i + 1 < argv.length) {
      const candidate = Number(argv[i + 1])
      if (!Number.isNaN(candidate)) {
        iterations = candidate
      }
      i += 1
    }
  }
  return { iterations }
}

export async function runLoopUi(iterations?: number) {
  const ink = withFullScreen(<LoopApp iterations={iterations} />)
  const hold = ink.waitUntilExit()
  ;(globalThis as { __dekaRuntimeHold?: Promise<void> }).__dekaRuntimeHold = hold
  await ink.start()
  await hold
}

const LoopApp = ({ iterations }: { iterations?: number }) => {
  const [tasks, setTasks] = useState<LoopTask[]>([])
  const [progress, setProgress] = useState<string[]>([])
  const [selected, setSelected] = useState(0)
  const [statusMessage, setStatusMessage] = useState<string | null>(null)
  const [isAdding, setIsAdding] = useState(false)
  const [addPhase, setAddPhase] = useState<AddPhase>('category')
  const [addBuffer, setAddBuffer] = useState('')
  const [draft, setDraft] = useState({ category: '', description: '', steps: '' })
  const { exit } = useApp()
  const { width: screenWidth, height: screenHeight } = useScreenSize()

  const refresh = async () => {
    try {
      const state = await loadLoopState()
      setTasks(state.tasks ?? [])
      const lines = await loadProgress()
      setProgress(lines)
      setStatusMessage(null)
    } catch (error) {
      setStatusMessage(error instanceof Error ? error.message : String(error))
    }
  }

  useEffect(() => {
    refresh()
    const interval = setInterval(() => refresh(), REFRESH_MS)
    return () => clearInterval(interval)
  }, [])

  useEffect(() => {
    if (selected >= tasks.length && tasks.length > 0) {
      setSelected(tasks.length - 1)
    }
    if (tasks.length === 0) {
      setSelected(0)
    }
  }, [tasks.length, selected])

  useInput((input, key) => {
    if (isAdding) {
      if (key.escape) {
        setIsAdding(false)
        setAddBuffer('')
        setStatusMessage('task entry cancelled')
        return
      }
      if (key.return) {
        const trimmed = addBuffer.trim()
        if (!trimmed) {
          setStatusMessage('field cannot be blank')
          return
        }
        if (addPhase === 'category') {
          setDraft((prev) => ({ ...prev, category: trimmed }))
          setAddPhase('description')
          setAddBuffer('')
          setStatusMessage('enter description and press return')
          return
        }
        if (addPhase === 'description') {
          setDraft((prev) => ({ ...prev, description: trimmed }))
          setAddPhase('steps')
          setAddBuffer('')
          setStatusMessage('enter steps separated by ";" and press return')
          return
        }
        if (addPhase === 'steps') {
          const finalDraft = { ...draft, steps: trimmed }
          setIsAdding(false)
          setAddPhase('category')
          setAddBuffer('')
          setDraft({ category: '', description: '', steps: '' })
          persistNewTask(finalDraft)
            .then(() => {
              refresh()
            })
            .catch((error) => {
              setStatusMessage(error instanceof Error ? error.message : String(error))
            })
          return
        }
      }
      if (key.backspace) {
        setAddBuffer((prev) => prev.slice(0, -1))
        return
      }
      if (input) {
        setAddBuffer((prev) => prev + input)
      }
      return
    }

    if (key.upArrow && tasks.length > 0) {
      setSelected((prev) => (prev === 0 ? prev : prev - 1))
      return
    }
    if (key.downArrow && tasks.length > 0) {
      setSelected((prev) => (prev + 1 >= tasks.length ? prev : prev + 1))
      return
    }
    if (input === 'q') {
      exit()
      return
    }
    if (input === 'r') {
      refresh()
      return
    }
    if (input === 'a') {
      setIsAdding(true)
      setAddPhase('category')
      setAddBuffer('')
      setDraft({ category: '', description: '', steps: '' })
      setStatusMessage('enter category and press return (ESC to cancel)')
      return
    }
    if (input === 's' && tasks.length > 0) {
      const current = tasks[selected]
      const nextStatus = STATUS_ORDER[(STATUS_ORDER.indexOf(current.status) + 1) % STATUS_ORDER.length]
      persistStatus(current.id, nextStatus)
        .then(() => {
          refresh()
        })
        .catch((error) => {
          setStatusMessage(error instanceof Error ? error.message : String(error))
        })
      return
    }
  })

  const selectedTask = tasks[selected]
  const loopSummary = useMemo(() => {
    const counts = { waiting: 0, running: 0, done: 0, failed: 0 }
    for (const task of tasks) {
      counts[task.status] += 1
    }
    return counts
  }, [tasks])

  const contentWidth = Math.max(10, screenWidth - OUTER_PADDING * 2 - 2)
  const maxListRows = Math.max(3, Math.min(tasks.length, Math.floor(screenHeight * 0.3)))
  const maxWindowStart = Math.max(0, tasks.length - maxListRows)
  const windowStart = Math.min(Math.max(0, selected - Math.max(0, maxListRows - 1)), maxWindowStart)
  const visibleTasks = tasks.slice(windowStart, windowStart + maxListRows)

  const iterationLabel = iterations ? `max ${iterations} iterations` : 'unlimited iterations'

  return (
    <Box flexDirection="column" paddingX={OUTER_PADDING} paddingY={1}>
      <Text bold>loop monitor ({iterationLabel})</Text>
      <Text dimColor>{tasks.length} task(s)</Text>
      <Text> </Text>
      <Text underline>tasks</Text>
      {tasks.length === 0 && <Text dimColor>no tasks yet</Text>}
      {visibleTasks.map((task, idx) => {
        const actualIndex = idx + windowStart
        const isSelected = actualIndex === selected
        const statusBadge = task.status.slice(0, 1).toUpperCase()
        const label = truncateText(normalizeInline(task.description), Math.max(10, contentWidth - 6))
        return (
          <Text key={task.id} color={isSelected ? 'green' : undefined} bold={isSelected}>
            {isSelected ? '> ' : '  '}
            {statusBadge} {label}
          </Text>
        )
      })}
      <Text> </Text>
      <Text underline>details</Text>
      {selectedTask ? (
        <>
          <Text bold>{truncateText(normalizeInline(selectedTask.description), contentWidth)}</Text>
          <Text dimColor>
            {truncateText(normalizeInline(`${selectedTask.category} • ${selectedTask.status}`), contentWidth)}
          </Text>
          <Text underline>steps</Text>
          {selectedTask.steps.length === 0 ? (
            <Text dimColor>no steps defined</Text>
          ) : (
            selectedTask.steps.map((step, idx) => (
              <Text key={idx}>- {truncateText(normalizeInline(step), Math.max(10, contentWidth - 2))}</Text>
            ))
          )}
          {progress.length > 0 && (
            <>
              <Text underline>progress</Text>
              {progress.map((line, idx) => (
                <Text key={`${line}-${idx}`}>{truncateText(normalizeLine(line), contentWidth)}</Text>
              ))}
            </>
          )}
        </>
      ) : (
        <Text dimColor>select a task to view details</Text>
      )}
      <Text> </Text>
      <Text dimColor>commands: a=add task, s=cycle status, r=refresh, q=quit</Text>
      <Text dimColor>
        status: waiting={loopSummary.waiting} running={loopSummary.running} done={loopSummary.done} failed={loopSummary.failed}
      </Text>
      {statusMessage && <Text color="yellow">{statusMessage}</Text>}
      {isAdding && <Text>entering {addPhase} &gt; {addBuffer}</Text>}
    </Box>
  )
}

function truncateText(text: string, max: number) {
  if (text.length <= max) return text
  if (max <= 3) return text.slice(0, max)
  return `${text.slice(0, max - 3)}...`
}

function normalizeInline(text: string) {
  return text
    .replace(/\r/g, '')
    .replace(/\n+/g, ' ')
    .replace(/\s+/g, ' ')
    .trim()
}

function normalizeLine(text: string) {
  return text.replace(/\r/g, '').replace(/\t/g, '  ').trimEnd()
}

async function loadLoopState(): Promise<LoopState> {
  try {
    const raw = await fs.readFile(LOOP_JSON, 'utf-8')
    return JSON.parse(raw) as LoopState
  } catch (error) {
    if ((error as { code?: string }).code === 'ENOENT') {
      return { tasks: [] }
    }
    throw error
  }
}

async function saveLoopState(state: LoopState) {
  await fs.mkdir(path.dirname(LOOP_JSON), { recursive: true })
  await fs.writeFile(LOOP_JSON, JSON.stringify(state, null, 2), 'utf-8')
}

async function loadProgress(): Promise<string[]> {
  try {
    const raw = await fs.readFile(PROGRESS_MD, 'utf-8')
    const lines = raw
      .split(/\r?\n/)
      .map((line) => line.replace(/\r/g, '').trimEnd())
      .filter((line) => line.trim().length > 0)
    return lines.slice(-8)
  } catch (error) {
    if ((error as { code?: string }).code === 'ENOENT') {
      return ['progress file missing']
    }
    throw error
  }
}

async function persistNewTask(task: { category: string; description: string; steps: string }) {
  const state = await loadLoopState()
  state.tasks.push({
    id: `task-${Date.now()}`,
    category: task.category,
    description: task.description,
    steps: task.steps
      .split(';')
      .map((step) => step.trim())
      .filter((step) => step.length > 0),
    status: 'waiting',
  })
  await saveLoopState(state)
}

async function persistStatus(id: string, status: TaskStatus) {
  const state = await loadLoopState()
  const idx = state.tasks.findIndex((task) => task.id === id)
  if (idx === -1) return
  state.tasks[idx].status = status
  await saveLoopState(state)
}

type AddPhase = 'category' | 'description' | 'steps'

const args = parseArgs(process.argv.slice(2))

runLoopUi(args.iterations).catch((error) => {
  console.error(error)
  process.exit(1)
})
