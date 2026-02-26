import { NextResponse } from 'next/server'
import { loadCodeExamples } from '@/lib/examples'

export async function GET() {
  try {
    const examples = await loadCodeExamples()
    return NextResponse.json(examples)
  } catch (error) {
    console.error('Error loading examples:', error)
    return NextResponse.json(
      { error: 'Failed to load examples' },
      { status: 500 }
    )
  }
}