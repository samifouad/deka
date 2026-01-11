'use client'

import { useState, useEffect } from 'react'
import { Button } from '@/components/ui/button'

const terminalCommands = [
  { command: 'tana new chain my-chain', output: '✓ Created new blockchain: my-chain' },
  { command: 'tana start', output: '✓ Ledger service running on port 8080' },
  { command: 'tana deploy contract ./token.ts', output: '✓ Contract deployed: ctr_token_123' },
  { command: 'tana balance @alice USD', output: 'Balance: 1,000 USD' },
  { command: 'tana transfer @alice @bob 100 USD', output: '✓ Transfer completed' }
]

export function AnimatedTerminal() {
  const [currentLine, setCurrentLine] = useState(0)
  const [currentChar, setCurrentChar] = useState(0)
  const [showCursor, setShowCursor] = useState(true)
  const [commandHistory, setCommandHistory] = useState<Array<{command: string, output: string}>>([])

  useEffect(() => {
    if (currentLine < terminalCommands.length) {
      const currentCommand = terminalCommands[currentLine].command
      
      if (currentChar < currentCommand.length) {
        const timeout = setTimeout(() => {
          setCurrentChar(currentChar + 1)
        }, 50)
        return () => clearTimeout(timeout)
      } else {
        // Show output after command is complete
        const timeout = setTimeout(() => {
          setCommandHistory(prev => [
            ...prev,
            { command: currentCommand, output: terminalCommands[currentLine].output }
          ])
          setCurrentLine(currentLine + 1)
          setCurrentChar(0)
        }, 1000)
        return () => clearTimeout(timeout)
      }
    }
  }, [currentLine, currentChar])

  useEffect(() => {
    const cursorInterval = setInterval(() => {
      setShowCursor(prev => !prev)
    }, 500)
    return () => clearInterval(cursorInterval)
  }, [])

  const getCurrentCommand = () => {
    if (currentLine < terminalCommands.length) {
      return terminalCommands[currentLine].command.slice(0, currentChar)
    }
    return ''
  }

  return (
    <div className="bg-card/95 backdrop-blur-sm border border-border rounded-lg p-6 font-mono text-sm shadow-2xl">
      <div className="flex items-center gap-2 mb-4">
        <div className="w-3 h-3 rounded-full bg-red-500"></div>
        <div className="w-3 h-3 rounded-full bg-yellow-500"></div>
        <div className="w-3 h-3 rounded-full bg-green-500"></div>
        <span className="text-muted-foreground ml-2">terminal</span>
      </div>
      
      <div className="space-y-2 min-h-[200px]">
        {commandHistory.map((item, index) => (
          <div key={index} className="space-y-1">
            <div className="text-foreground">
              <span className="text-primary">$</span> {item.command}
            </div>
            <div className="text-muted-foreground ml-4">{item.output}</div>
          </div>
        ))}

        {currentLine < terminalCommands.length && (
          <div className="text-foreground">
            <span className="text-primary">$</span> {getCurrentCommand()}
            {showCursor && <span className="text-foreground animate-pulse">_</span>}
          </div>
        )}
      </div>
      
      {currentLine >= terminalCommands.length && (
        <div className="mt-4 pt-4 border-t border-border">
          <Button size="sm" className="bg-primary hover:bg-primary/90 text-primary-foreground font-bold border border-primary/30">
            Try tana CLI
          </Button>
        </div>
      )}
    </div>
  )
}