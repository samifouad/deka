'use client'

export function BlueprintGrid() {
  return (
    <div className="fixed inset-0 pointer-events-none overflow-hidden">
      <svg className="absolute inset-0 w-full h-full" xmlns="http://www.w3.org/2000/svg">
        <defs>
          <pattern
            id="grid"
            width="40"
            height="40"
            patternUnits="userSpaceOnUse"
          >
            <path
              d="M 40 0 L 0 0 0 40"
              fill="none"
              stroke="hsl(var(--border))"
              strokeWidth="1"
              opacity="0.3"
            />
          </pattern>
          <pattern
            id="grid-large"
            width="200"
            height="200"
            patternUnits="userSpaceOnUse"
          >
            <path
              d="M 200 0 L 0 0 0 200"
              fill="none"
              stroke="hsl(var(--border))"
              strokeWidth="1"
              opacity="0.4"
            />
          </pattern>
        </defs>
        <rect width="100%" height="100%" fill="url(#grid)" />
        <rect width="100%" height="100%" fill="url(#grid-large)" />
        
        {/* Animated grid lines */}
        <g opacity="0.15">
          <line
            x1="0"
            y1="25%"
            x2="100%"
            y2="25%"
            stroke="hsl(var(--primary))"
            strokeWidth="1"
            strokeDasharray="5, 10"
            opacity="0.5"
          >
            <animate
              attributeName="stroke-dashoffset"
              from="0"
              to="15"
              dur="20s"
              repeatCount="indefinite"
            />
          </line>
          <line
            x1="0"
            y1="50%"
            x2="100%"
            y2="50%"
            stroke="hsl(var(--primary))"
            strokeWidth="1"
            strokeDasharray="5, 10"
            opacity="0.5"
          >
            <animate
              attributeName="stroke-dashoffset"
              from="0"
              to="15"
              dur="25s"
              repeatCount="indefinite"
            />
          </line>
          <line
            x1="0"
            y1="75%"
            x2="100%"
            y2="75%"
            stroke="hsl(var(--primary))"
            strokeWidth="1"
            strokeDasharray="5, 10"
            opacity="0.5"
          >
            <animate
              attributeName="stroke-dashoffset"
              from="0"
              to="15"
              dur="30s"
              repeatCount="indefinite"
            />
          </line>
        </g>
        
        {/* Corner decorations */}
        <g opacity="0.2">
          {/* Top left */}
          <path
            d="M 0 0 L 100 0 L 0 100"
            fill="none"
            stroke="hsl(var(--primary))"
            strokeWidth="2"
            opacity="0.3"
          />
          {/* Top right */}
          <path
            d="M 100% 0 L 100% -100 L 100% 100"
            fill="none"
            stroke="hsl(var(--primary))"
            strokeWidth="2"
            opacity="0.3"
            transform="translate(-100, 0)"
          />
          {/* Bottom left */}
          <path
            d="M 0 100% L 100 100% L 0 100%"
            fill="none"
            stroke="hsl(var(--primary))"
            strokeWidth="2"
            opacity="0.3"
            transform="translate(0, -100)"
          />
          {/* Bottom right */}
          <path
            d="M 100% 100% L 100% 100 L 100% 100%"
            fill="none"
            stroke="hsl(var(--primary))"
            strokeWidth="2"
            opacity="0.3"
            transform="translate(-100, -100)"
          />
        </g>
      </svg>
    </div>
  )
}