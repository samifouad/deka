import React, { type ComponentPropsWithoutRef, forwardRef, useCallback, useEffect, useState } from 'react'
import { Box, type DOMElement, render, useInput, useStdout } from 'ink'

type BoxProps = ComponentPropsWithoutRef<typeof Box>

export function useScreenSize() {
  const { stdout } = useStdout()
  const getSize = useCallback(() => {
    const height = typeof stdout?.rows === 'number' && stdout.rows > 0 ? stdout.rows : 24
    const width = typeof stdout?.columns === 'number' && stdout.columns > 0 ? stdout.columns : 80
    return { height, width }
  }, [stdout])
  const [size, setSize] = useState(getSize)

  useEffect(() => {
    const onResize = () => setSize(getSize())
    stdout.on('resize', onResize)
    return () => stdout.off('resize', onResize)
  }, [stdout, getSize])

  return size
}

export const FullScreenBox = forwardRef<DOMElement, BoxProps>(function FullScreenBox(props, ref) {
  useInput(() => {})
  const { height, width } = useScreenSize()
  return <Box ref={ref} height={height} width={width} {...props} />
})

async function write(content: string) {
  return new Promise<void>((resolve, reject) => {
    process.stdout.write(content, (error) => {
      if (error) reject(error)
      else resolve()
    })
  })
}

export function withFullScreen(node: React.ReactNode) {
  const instance = render(null)

  const waitUntilExit = (async () => {
    await instance.waitUntilExit()
    await write('\x1b[?1049l')
  })()

  return {
    instance,
    start: async () => {
      await write('\x1b[?1049h')
      instance.rerender(<FullScreenBox>{node}</FullScreenBox>)
    },
    waitUntilExit: () => waitUntilExit,
  }
}
