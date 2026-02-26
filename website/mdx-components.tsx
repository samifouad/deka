import type { MDXComponents } from 'mdx/types'
import Link from 'next/link'

// Direct export for server components
export const mdxComponents: MDXComponents = {
  h1: ({ children, id }) => (
    <h1 id={id} className="text-4xl font-bold text-foreground mt-8 mb-4 scroll-mt-20">
      {children}
    </h1>
  ),
    h2: ({ children, id }) => (
      <h2 id={id} className="text-3xl font-bold text-foreground mt-8 mb-4 scroll-mt-20">
        {children}
      </h2>
    ),
    h3: ({ children, id }) => (
      <h3 id={id} className="text-2xl font-bold text-foreground mt-6 mb-3 scroll-mt-20">
        {children}
      </h3>
    ),
    h4: ({ children, id }) => (
      <h4 id={id} className="text-xl font-bold text-foreground mt-4 mb-2 scroll-mt-20">
        {children}
      </h4>
    ),
    p: ({ children }) => (
      <p className="text-muted-foreground mb-4 leading-relaxed">
        {children}
      </p>
    ),
    ul: ({ children }) => (
      <ul className="list-disc list-inside text-muted-foreground mb-4 space-y-2">
        {children}
      </ul>
    ),
    ol: ({ children }) => (
      <ol className="list-decimal list-inside text-muted-foreground mb-4 space-y-2">
        {children}
      </ol>
    ),
    li: ({ children }) => (
      <li className="ml-4">{children}</li>
    ),
    a: ({ href, children }) => (
      <Link
        href={href || '#'}
        className="text-primary hover:text-primary/80 underline underline-offset-2"
      >
        {children}
      </Link>
    ),
    code: ({ children, className }) => {
      // Inline code
      if (!className) {
        return (
          <code className="bg-secondary px-1.5 py-0.5 rounded text-sm text-primary font-mono">
            {children}
          </code>
        )
      }
      // Block code (handled by rehype-pretty-code)
      return <code className={className}>{children}</code>
    },
    pre: ({ children }) => (
      <pre className="bg-secondary/50 border border-border rounded-lg p-4 overflow-x-auto my-4 text-sm">
        {children}
      </pre>
    ),
    blockquote: ({ children }) => (
      <blockquote className="border-l-4 border-primary/30 pl-4 my-4 italic text-muted-foreground">
        {children}
      </blockquote>
    ),
    table: ({ children }) => (
      <div className="overflow-x-auto my-4">
        <table className="w-full border-collapse border border-border">
          {children}
        </table>
      </div>
    ),
    th: ({ children }) => (
      <th className="border border-border bg-secondary px-4 py-2 text-left font-semibold text-foreground">
        {children}
      </th>
    ),
    td: ({ children }) => (
      <td className="border border-border px-4 py-2 text-muted-foreground">
        {children}
      </td>
    ),
}

// Hook export for client components compatibility
export function useMDXComponents(components: MDXComponents): MDXComponents {
  return {
    ...mdxComponents,
    ...components,
  }
}
