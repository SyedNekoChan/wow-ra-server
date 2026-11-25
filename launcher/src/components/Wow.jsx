// Launcher UI kit built on the textures in src/assets/ui (see SOURCES.txt). Styles live in index.css.
import indicatorGreen from '../assets/ui/indicator-green.png'
import indicatorGray from '../assets/ui/indicator-gray.png'
import indicatorYellow from '../assets/ui/indicator-yellow.png'

const INDICATOR = { on: indicatorGreen, off: indicatorGray, wait: indicatorYellow }

// Small original mark for Emberfall: an ember inside a cut diamond, in ink.
export function Emblem({ className = '' }) {
  return (
    <svg viewBox="0 0 32 32" className={className} aria-hidden="true">
      <path d="M16 1.5 L30.5 16 L16 30.5 L1.5 16 Z" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinejoin="round" />
      <path d="M16 6 L26 16 L16 26 L6 16 Z" fill="none" stroke="currentColor" strokeWidth=".8" opacity=".55" />
      <path d="M16 8.5 C19.5 12 21 14.6 21 17.4 C21 20.4 18.8 22.6 16 22.6 C13.2 22.6 11 20.4 11 17.6 C11 15.6 12.1 14.1 13.4 13 C13.4 14.8 14.2 15.9 15.2 16.3 C14.6 13.4 15.1 10.9 16 8.5 Z" fill="#b8471a" />
      <path d="M16 15 C17.6 16.6 18.3 17.7 18.3 18.9 C18.3 20.2 17.3 21.1 16 21.1 C14.7 21.1 13.7 20.2 13.7 18.9 C13.7 17.6 14.6 16.4 16 15 Z" fill="#e8893a" />
    </svg>
  )
}

// on: green, off: grey, wait: yellow (COMMON/Indicator-*)
export function Indicator({ on, tone, className = 'w-4 h-4' }) {
  return <img src={INDICATOR[tone || (on ? 'on' : 'off')]} alt="" className={className} draggable="false" />
}

export function StatusChip({ on, children }) {
  return (
    <span className="chip">
      <Indicator on={on} className="w-3.5 h-3.5" />
      {children}
    </span>
  )
}

// Text navigation in ink with a drawn underline.
export function NavLink({ children, ...rest }) {
  return (
    <button className="nav-link" {...rest}>
      {children}
    </button>
  )
}

export function Button({ className = '', children, ...rest }) {
  return (
    <button className={`wow-btn ${className}`} {...rest}>
      <span>{children}</span>
    </button>
  )
}

export function BigButton({ className = '', children, ...rest }) {
  return (
    <button className={`wow-big ${className}`} {...rest}>
      <span>{children}</span>
    </button>
  )
}

export function CloseButton({ className = '', label = 'Close', ...rest }) {
  return <button className={`wow-x ${className}`} aria-label={label} title={label} {...rest} />
}

export function HideButton({ className = '', label = 'Minimize', ...rest }) {
  return <button className={`wow-hide ${className}`} aria-label={label} title={label} {...rest} />
}

export function Check({ checked, onChange, ...rest }) {
  return (
    <span className="wow-check">
      <input type="checkbox" checked={checked} onChange={onChange} {...rest} />
    </span>
  )
}

// Glue loading bar: border texture over an ember status-bar fill with the spark at its edge.
export function Bar({ pct, label }) {
  return (
    <div className="wow-bar" role="progressbar" aria-valuemin={0} aria-valuemax={100} aria-valuenow={Math.round(pct)} aria-label={label}>
      <div className="wow-bar-track">
        <div className="wow-bar-fill" style={{ width: `${pct}%` }} />
        {pct > 0 && pct < 100 && <div className="wow-bar-spark" style={{ left: `${pct}%` }} />}
      </div>
    </div>
  )
}

// Dialog: grey stone border around a quest-log page, with the stone title plaque.
export function Panel({ title, className = '', children, ...rest }) {
  return (
    <div className={`wow-frame ${className}`} {...rest}>
      {title && <div className="wow-header">{title}</div>}
      {children}
    </div>
  )
}
