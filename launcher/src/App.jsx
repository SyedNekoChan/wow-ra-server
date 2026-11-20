import { useCallback, useEffect, useMemo, useState } from 'react'
import { invoke, listen, pickFolder } from './api.js'

const CLASS_COLORS = {
  1: '#c79c6e', 2: '#f58cba', 3: '#abd473', 4: '#fff569', 5: '#f0f0f0',
  6: '#c41f3b', 7: '#0070de', 8: '#69ccf0', 9: '#8bbf5b', 11: '#ff7d0a',
}

const mb = (n) => (n / 1048576).toFixed(n > 1073741824 ? 0 : 1)
const asText = (e) => (typeof e === 'string' ? e : e?.message || JSON.stringify(e))

function Embers() {
  const embers = useMemo(
    () =>
      Array.from({ length: 22 }, (_, i) => ({
        id: i,
        left: Math.random() * 100,
        size: 3 + Math.random() * 5,
        dur: 9 + Math.random() * 12,
        delay: -Math.random() * 18,
        dx: (Math.random() - 0.5) * 160,
      })),
    [],
  )
  return (
    <div className="absolute inset-0 overflow-hidden pointer-events-none" aria-hidden="true">
      {embers.map((e) => (
        <span
          key={e.id}
          className="ember"
          style={{ left: `${e.left}%`, width: e.size, height: e.size, animationDuration: `${e.dur}s`, animationDelay: `${e.delay}s`, '--dx': `${e.dx}px` }}
        />
      ))}
    </div>
  )
}

function Emblem({ className = '' }) {
  return (
    <svg viewBox="0 0 64 64" className={className} aria-hidden="true">
      <defs>
        <linearGradient id="g" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" stopColor="#f3d27a" />
          <stop offset="1" stopColor="#a67a25" />
        </linearGradient>
      </defs>
      <path d="M32 4 L56 12 L53 36 L32 60 L11 36 L8 12 Z" fill="#17120a" stroke="url(#g)" strokeWidth="3" strokeLinejoin="round" />
      <path d="M32 16 L36 29 L49 32 L36 35 L32 48 L28 35 L15 32 L28 29 Z" fill="url(#g)" />
      <circle cx="32" cy="32" r="4" fill="#17120a" />
    </svg>
  )
}

function Panel({ title, right, children, className = '' }) {
  return (
    <section className={`panel p-5 ${className}`}>
      {(title || right) && (
        <div className="flex items-center justify-between mb-3">
          <h2 className="font-display font-semibold tracking-[.18em] text-[13px] text-gold uppercase">{title}</h2>
          {right}
        </div>
      )}
      {children}
    </section>
  )
}

function Modal({ title, onClose, children, wide }) {
  return (
    <div className="fixed inset-0 z-40 flex items-center justify-center bg-black/70 backdrop-blur-sm" onMouseDown={(e) => e.target === e.currentTarget && onClose?.()}>
      <div className={`panel p-6 ${wide ? 'w-[560px]' : 'w-[460px]'} max-h-[90vh] overflow-auto`}>
        <div className="flex items-center justify-between mb-4">
          <h2 className="font-display font-semibold tracking-[.16em] text-gold uppercase text-lg">{title}</h2>
          {onClose && (
            <button className="btn-ghost rounded-lg w-8 h-8 text-dim" onClick={onClose} aria-label="Close">
              ✕
            </button>
          )}
        </div>
        {children}
      </div>
    </div>
  )
}

function Chip({ tone = 'dim', children }) {
  const tones = { ok: 'text-ok border-ok/40 bg-ok/10', bad: 'text-bad border-bad/40 bg-bad/10', gold: 'text-gold border-gold/40 bg-gold/10', dim: 'text-dim border-edge bg-black/30' }
  return <span className={`inline-flex items-center gap-1.5 text-[11px] font-medium tracking-wide px-2.5 py-1 rounded-full border ${tones[tone]}`}>{children}</span>
}

function CharacterCard({ c }) {
  const color = CLASS_COLORS[c.class_id] || '#c9b88a'
  return (
    <li className="card flex items-center gap-3 rounded-xl border border-edge bg-black/30 pl-0 pr-3 py-2.5 overflow-hidden">
      <span className="self-stretch w-1.5 rounded-r" style={{ background: color }} />
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="font-display font-semibold text-parch truncate">{c.name}</span>
          {c.online && <span className="w-2 h-2 rounded-full bg-ok" title="Online" />}
        </div>
        <div className="text-xs text-dim truncate">
          Level {c.level} {c.race} <span style={{ color }}>{c.class}</span>
        </div>
      </div>
      <div className="text-right text-[11px] leading-tight text-dim">
        <div className="text-gold">✦ {c.achievements} realm achievement{c.achievements === 1 ? '' : 's'}</div>
        <div>{c.playtime_hours.toFixed(1)} h played</div>
      </div>
    </li>
  )
}

export default function App() {
  const [status, setStatus] = useState(null)
  const [update, setUpdate] = useState(null)
  const [session, setSession] = useState(null) // { username, gm_level, characters, password } (password stays in memory only)
  const [busy, setBusy] = useState('') // '', setup, update, login, play
  const [progress, setProgress] = useState(null)
  const [phase, setPhase] = useState(null)
  const [msg, setMsg] = useState(null) // { tone, text }
  const [modal, setModal] = useState(null)
  const [form, setForm] = useState({ user: '', password: '' })

  const say = (tone, text) => setMsg({ tone, text })
  const refresh = useCallback(async () => {
    const s = await invoke('get_status')
    setStatus(s)
    return s
  }, [])
  const recheck = useCallback(async () => {
    setUpdate(await invoke('check_update'))
  }, [])

  useEffect(() => {
    refresh().then(() => recheck())
    const subs = [
      listen('progress', (e) => setProgress(e.payload)),
      listen('play', (e) => setPhase(e.payload)),
    ]
    return () => subs.forEach((p) => p.then((off) => off()))
  }, [refresh, recheck])

  async function run(kind, fn) {
    setBusy(kind)
    setMsg(null)
    try {
      await fn()
    } catch (e) {
      say('bad', asText(e))
    } finally {
      setBusy('')
    }
  }

  const chooseClient = async () => {
    const p = await pickFolder('Select your World of Warcraft 3.3.5a folder (the one with Wow.exe)')
    if (!p) return
    try {
      setStatus(await invoke('set_client_folder', { path: p }))
      say('ok', 'Game folder saved.')
    } catch (e) {
      say('bad', asText(e))
    }
  }
  const chooseServer = async () => {
    const p = await pickFolder('Select the folder for the server files')
    if (!p) return
    try {
      setStatus(await invoke('set_server_folder', { path: p }))
      recheck()
    } catch (e) {
      say('bad', asText(e))
    }
  }

  const doSetup = () =>
    run('setup', async () => {
      setProgress(null)
      await invoke('run_setup')
      await refresh()
      await recheck()
      say('ok', 'Installed. Create an account, log in and press Play.')
    })
  const doUpdate = () =>
    run('update', async () => {
      setProgress(null)
      await invoke('apply_update')
      await refresh()
      await recheck()
      say('ok', 'Updated to the newest version.')
    })
  const doLogin = (e) => {
    e?.preventDefault()
    return run('login', async () => {
      const r = await invoke('login', { user: form.user.trim(), password: form.password })
      setSession({ ...r, password: form.password })
      setForm({ user: '', password: '' })
    })
  }
  const doPlay = () =>
    run('play', async () => {
      setPhase({ phase: 'starting', message: 'Starting ...' })
      try {
        await invoke('play', { user: session.username, password: session.password })
      } finally {
        setPhase(null)
        await refresh()
        try {
          const chars = await invoke('characters', { user: session.username })
          setSession((s) => (s ? { ...s, characters: chars } : s))
        } catch {}
      }
    })

  if (!status) {
    return <div className="stage h-full grid place-items-center text-dim font-display tracking-widest">LOADING ...</div>
  }

  // ----- the one big button -----
  let action
  if (!status.setup_done) action = { label: 'Install', hint: 'Downloads the realm files (about 1.4 GB, once).', run: doSetup, on: !busy }
  else if (update?.available) action = { label: 'Update', hint: `Version ${update.latest} is ready (you have ${update.installed || 'none'}).`, run: doUpdate, on: !busy }
  else if (!session) action = { label: 'Log in to play', hint: 'Log in on the left, or create an account.', run: null, on: false }
  else action = { label: 'Play', hint: `Logged in as ${session.username}. The game opens at the login and realm screens.`, run: doPlay, on: !busy }
  if (busy === 'play') action = { ...action, label: phase?.phase === 'ingame' ? 'In game' : 'Starting', on: false }
  if (busy === 'setup') action = { ...action, label: 'Installing', on: false }
  if (busy === 'update') action = { ...action, label: 'Updating', on: false }

  const showBar = (busy === 'setup' || busy === 'update') && progress
  const pct = Math.min(100, Math.max(0, progress?.overall ?? 0))

  return (
    <div className="stage relative h-full flex flex-col text-parch">
      <div className="grain" />
      <Embers />

      {/* header */}
      <header className="relative z-10 flex items-center justify-between px-8 pt-6 pb-3">
        <div className="flex items-center gap-3">
          <Emblem className="w-11 h-11 drop-shadow-[0_0_12px_rgba(212,168,75,.45)]" />
          <div>
            <h1 className="font-display font-extrabold text-3xl tracking-[.14em] title-shimmer leading-none">RA REALM</h1>
            <p className="text-[11px] tracking-[.3em] text-dim mt-1 uppercase">Wrath of the Lich King · 3.3.5a · Achievements Realm</p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <Chip tone={status.mysql_running ? 'ok' : 'dim'}>● Database {status.mysql_running ? 'on' : 'off'}</Chip>
          <Chip tone={status.server_running ? 'ok' : 'dim'}>● Realm {status.server_running ? 'online' : 'offline'}</Chip>
          <button className="btn-ghost rounded-lg px-3 h-9 text-sm" onClick={() => setModal('accounts')}>Accounts</button>
          <button className="btn-ghost rounded-lg px-3 h-9 text-sm" onClick={() => setModal('settings')}>Settings</button>
        </div>
      </header>
      <div className="rune-line mx-8" />

      <main className="relative z-10 flex-1 min-h-0 grid grid-cols-[400px_1fr] gap-6 px-8 py-5">
        {/* left: account and characters */}
        <Panel
          title={session ? 'Your characters' : 'Account login'}
          right={session && <button className="text-xs text-dim hover:text-gold transition-colors" onClick={() => setSession(null)}>Log out</button>}
          className="flex flex-col min-h-0"
        >
          {!session ? (
            <form onSubmit={doLogin} className="flex flex-col gap-3 mt-1">
              <label className="text-xs text-dim">
                Account name
                <input className="field mt-1" autoComplete="off" value={form.user} onChange={(e) => setForm({ ...form, user: e.target.value })} />
              </label>
              <label className="text-xs text-dim">
                Password
                <input className="field mt-1" type="password" autoComplete="off" value={form.password} onChange={(e) => setForm({ ...form, password: e.target.value })} />
              </label>
              <button className="btn-ghost rounded-xl h-11 font-semibold text-gold mt-1" disabled={busy || !form.user || !form.password || !status.setup_done}>
                {busy === 'login' ? 'Checking ...' : 'Log in'}
              </button>
              {!status.setup_done && <p className="text-xs text-dim">Install the realm files first, then you can log in.</p>}
              <button type="button" className="text-xs text-dim hover:text-gold transition-colors text-left" onClick={() => setModal('accounts')}>
                No account yet? Create one →
              </button>
            </form>
          ) : (
            <div className="flex flex-col min-h-0 flex-1">
              <div className="flex items-center gap-2 mb-3">
                <span className="font-display text-xl text-parch tracking-wider">{session.username}</span>
                {session.gm_level > 0 && <Chip tone="gold">GM {session.gm_level}</Chip>}
              </div>
              {session.gm_level > 0 && <p className="text-[11px] text-dim mb-2">GM accounts never earn achievements. Use a normal account to test them.</p>}
              {session.characters.length === 0 ? (
                <p className="text-sm text-dim">No characters yet. Press Play and create one at the character screen.</p>
              ) : (
                <ul className="flex flex-col gap-2 overflow-auto pr-1 flex-1 min-h-0">
                  {session.characters.map((c) => (
                    <CharacterCard key={c.name} c={c} />
                  ))}
                </ul>
              )}
            </div>
          )}
        </Panel>

        {/* right: hero and the big button */}
        <section className="panel p-8 flex flex-col justify-between min-h-0 relative overflow-hidden">
          <div className="absolute -right-24 -top-24 w-80 h-80 rounded-full bg-gold/10 blur-3xl pointer-events-none" />
          <div>
            <p className="text-[11px] tracking-[.35em] text-gold uppercase">Free · No shop · Earn everything</p>
            <h2 className="font-display font-extrabold text-4xl leading-tight mt-3 max-w-[26ch]">Every achievement is earned in Azeroth.</h2>
            <p className="text-dim mt-3 max-w-[52ch] text-[15px] leading-relaxed">
              A local test realm for hardcore-style achievements. Log in, press Play, and the launcher starts the realm, opens the game and stops everything when you quit.
            </p>
          </div>

          <div className="space-y-4">
            {showBar && (
              <div>
                <div className="flex items-baseline justify-between text-sm mb-2">
                  <span className="text-parch font-medium">{progress.label}</span>
                  <span className="text-gold tabular-nums">{pct.toFixed(0)}%</span>
                </div>
                <div className="h-3 rounded-full bg-black/60 border border-edge overflow-hidden">
                  <div className="bar-fill h-full rounded-full" style={{ width: `${pct}%` }} />
                </div>
                <p className="text-xs text-dim mt-2 min-h-[1.1rem]">
                  {progress.total > 0 && progress.stage !== 'package' && progress.done > 1000 ? `${mb(progress.done)} of ${mb(progress.total)} MB · ` : ''}
                  {progress.note}
                </p>
              </div>
            )}
            {busy === 'play' && phase && (
              <div className="flex items-center gap-3 text-sm">
                <span className="w-2.5 h-2.5 rounded-full bg-gold animate-pulse" />
                <span>{phase.message}</span>
              </div>
            )}
            {msg && (
              <p className={`text-sm rounded-lg border px-3 py-2 ${msg.tone === 'bad' ? 'border-bad/50 bg-bad/10 text-bad' : 'border-ok/40 bg-ok/10 text-ok'}`}>{msg.text}</p>
            )}

            <div className="flex items-end gap-4">
              <button className="btn-play rounded-2xl h-16 px-10 min-w-[250px] whitespace-nowrap font-display font-extrabold text-xl tracking-[.2em] uppercase" disabled={!action.on} onClick={action.run}>
                {action.label}
              </button>
              <p className="text-xs text-dim max-w-[34ch] pb-1">{action.hint}</p>
            </div>
          </div>
        </section>
      </main>

      <footer className="relative z-10 px-8 pb-4 flex items-center justify-between text-[11px] text-dim">
        <span>Server files {status.installed_version ? `v${status.installed_version}` : 'not installed'} · Launcher v{status.launcher_version}</span>
        <span className="truncate max-w-[60%]">Game folder: {status.client_dir || 'not set'}</span>
      </footer>

      {/* first run: point to the game folder */}
      {!status.client_dir && (
        <Modal title="Welcome, adventurer">
          <p className="text-sm text-dim leading-relaxed">
            First, point me to your World of Warcraft <b className="text-parch">3.3.5a (English)</b> game folder: the one that contains <code className="text-gold">Wow.exe</code> and the <code className="text-gold">Data</code> folder.
            I never download the game itself.
          </p>
          {msg && <p className="text-sm text-bad mt-3">{msg.text}</p>}
          <button className="btn-play rounded-xl h-12 w-full mt-5 font-display font-bold tracking-[.18em] uppercase" onClick={chooseClient}>
            Choose game folder
          </button>
        </Modal>
      )}

      {modal === 'settings' && (
        <Modal title="Settings" onClose={() => setModal(null)} wide>
          <div className="space-y-4 text-sm">
            <div>
              <div className="text-xs text-dim mb-1">Game folder (3.3.5a)</div>
              <div className="flex gap-2">
                <div className="field truncate">{status.client_dir || 'not set'}</div>
                <button className="btn-ghost rounded-lg px-4" onClick={chooseClient}>Change</button>
              </div>
            </div>
            <div>
              <div className="text-xs text-dim mb-1">Server files folder</div>
              <div className="flex gap-2">
                <div className="field truncate">{status.server_dir}</div>
                <button className="btn-ghost rounded-lg px-4" onClick={chooseServer} disabled={!!busy}>Change</button>
              </div>
              <p className="text-[11px] text-dim mt-1">Point this at an existing server folder (for example the one you already use) to reuse it.</p>
            </div>
            <label className="flex items-center gap-3 cursor-pointer">
              <input type="checkbox" className="w-4 h-4 accent-[#d4a84b]" checked={status.autologin} onChange={async (e) => setStatus(await invoke('set_autologin', { on: e.target.checked }))} />
              <span>Type my login into the game for me <span className="text-dim">(experimental; if it misses, type it yourself)</span></span>
            </label>
            <div className="flex items-center gap-3 pt-2">
              <button className="btn-ghost rounded-lg px-4 h-10" onClick={recheck}>Check for updates</button>
              <button
                className="btn-ghost rounded-lg px-4 h-10"
                disabled={!!busy}
                onClick={() => run('stop', async () => { await invoke('stop_realm'); await refresh(); say('ok', 'Realm stopped.') })}
              >
                Stop the realm
              </button>
              <span className="text-xs text-dim">{update?.error ? update.error : update?.latest ? `Newest: v${update.latest}` : ''}</span>
            </div>
          </div>
        </Modal>
      )}

      {modal === 'accounts' && <AccountsModal status={status} onClose={() => setModal(null)} onCreated={(u) => setForm({ user: u, password: '' })} />}
    </div>
  )
}

function AccountsModal({ status, onClose, onCreated }) {
  const [tab, setTab] = useState('create')
  const [f, setF] = useState({ user: '', pass: '', gm: false })
  const [out, setOut] = useState(null)
  const [working, setWorking] = useState(false)

  async function submit(e) {
    e.preventDefault()
    setWorking(true)
    setOut(null)
    try {
      if (tab === 'create') {
        await invoke('create_account', { user: f.user.trim(), password: f.pass, gm: f.gm })
        setOut({ ok: true, text: `Account ${f.user.trim().toUpperCase()} created.` })
        onCreated(f.user.trim())
      } else {
        await invoke('delete_account', { user: f.user.trim(), password: f.pass })
        setOut({ ok: true, text: `Account ${f.user.trim().toUpperCase()} deleted.` })
      }
      setF({ user: '', pass: '', gm: f.gm })
    } catch (err) {
      setOut({ ok: false, text: asText(err) })
    } finally {
      setWorking(false)
    }
  }

  return (
    <Modal title="Accounts" onClose={onClose}>
      <div className="flex gap-2 mb-4">
        {['create', 'delete'].map((t) => (
          <button key={t} onClick={() => { setTab(t); setOut(null) }} className={`rounded-lg px-4 h-9 text-sm font-medium capitalize btn-ghost ${tab === t ? '!border-gold text-gold' : 'text-dim'}`}>
            {t} account
          </button>
        ))}
      </div>
      {!status.setup_done ? (
        <p className="text-sm text-dim">Install the realm files first. Accounts live in the realm database.</p>
      ) : (
        <form onSubmit={submit} className="flex flex-col gap-3">
          <label className="text-xs text-dim">
            Account name (2 to 16 letters or numbers)
            <input className="field mt-1" autoComplete="off" value={f.user} onChange={(e) => setF({ ...f, user: e.target.value })} />
          </label>
          <label className="text-xs text-dim">
            Password {tab === 'delete' && '(to confirm)'}
            <input className="field mt-1" type="password" autoComplete="off" value={f.pass} onChange={(e) => setF({ ...f, pass: e.target.value })} />
          </label>
          {tab === 'create' ? (
            <label className="flex items-center gap-3 text-sm cursor-pointer">
              <input type="checkbox" className="w-4 h-4 accent-[#d4a84b]" checked={f.gm} onChange={(e) => setF({ ...f, gm: e.target.checked })} />
              <span>Make this a GM account (level 3) <span className="text-dim">for testing commands. GMs do not earn achievements.</span></span>
            </label>
          ) : (
            <p className="text-xs text-dim">An account that still has characters cannot be deleted. Delete its characters in the game first.</p>
          )}
          <button className={`rounded-xl h-11 font-semibold mt-1 ${tab === 'create' ? 'btn-play' : 'btn-ghost text-bad'}`} disabled={working || !f.user || !f.pass}>
            {working ? 'Working ...' : tab === 'create' ? 'Create account' : 'Delete account'}
          </button>
          {out && <p className={`text-sm rounded-lg border px-3 py-2 ${out.ok ? 'border-ok/40 bg-ok/10 text-ok' : 'border-bad/50 bg-bad/10 text-bad'}`}>{out.text}</p>}
        </form>
      )}
    </Modal>
  )
}
