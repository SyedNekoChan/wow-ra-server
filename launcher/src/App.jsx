import { useCallback, useEffect, useState } from 'react'
import { invoke, listen, pickFolder, minimizeWindow, closeWindow } from './api.js'
import { BigButton, Bar, Button, Check, CloseButton, Emblem, HideButton, Indicator, NavLink, Panel, StatusChip } from './components/Wow.jsx'
import hero from './assets/ui/hero.webp'

const mb = (n) => (n / 1048576).toFixed(n > 1073741824 ? 0 : 1)
const asText = (e) => (typeof e === 'string' ? e : e?.message || JSON.stringify(e))

function Modal({ title, onClose, children, width = 440 }) {
  return (
    <div className="modal-veil fixed inset-0 z-40 flex items-center justify-center" onMouseDown={(e) => e.target === e.currentTarget && onClose?.()}>
      <div role="dialog" aria-modal="true" aria-label={title} className="modal-pop" style={{ width }}>
        <Panel title={title} className="pt-7 pb-2 px-3">
          {onClose && <CloseButton className="!absolute -right-[27px] -top-[27px] z-10" onClick={onClose} />}
          {children}
        </Panel>
      </div>
    </div>
  )
}

function Field({ label, children }) {
  return (
    <label className="block">
      <span className="label">{label}</span>
      {children}
    </label>
  )
}

export default function App() {
  const [status, setStatus] = useState(null)
  const [update, setUpdate] = useState(null)
  const [busy, setBusy] = useState('') // '', setup, update, play, stop
  const [progress, setProgress] = useState(null)
  const [phase, setPhase] = useState(null)
  const [msg, setMsg] = useState(null) // { tone, text }
  const [modal, setModal] = useState(null)

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
    const p = await pickFolder('Select your 3.3.5a game folder (the one with Wow.exe)')
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
      say('ok', 'Installed.')
    })
  const doUpdate = () =>
    run('update', async () => {
      setProgress(null)
      await invoke('apply_update')
      await refresh()
      await recheck()
      say('ok', 'Updated.')
    })
  const doPlay = () =>
    run('play', async () => {
      setPhase({ phase: 'starting', message: 'Starting ...' })
      try {
        await invoke('play')
      } finally {
        setPhase(null)
        await refresh()
      }
    })

  if (!status) {
    return (
      <div className="stage grid place-items-center">
        <div className="stage-paper" />
        <div className="stage-burn" />
        <Emblem className="w-12 h-12 text-ink" />
      </div>
    )
  }

  // ----- the one big button -----
  let action
  if (!status.setup_done) action = { label: 'Install', state: 'Not installed', detail: 'About 1.4 GB', run: doSetup, on: !busy }
  else if (update?.available) action = { label: 'Update', state: 'Update available', detail: `v${update.latest} · installed v${update.installed || 'none'}`, run: doUpdate, on: !busy }
  else action = { label: 'Play', state: 'Ready', detail: '', run: doPlay, on: !busy }
  if (busy === 'play') action = { ...action, label: phase?.phase === 'ingame' ? 'In game' : 'Starting', on: false }
  if (busy === 'setup') action = { ...action, label: 'Installing', on: false }
  if (busy === 'update') action = { ...action, label: 'Updating', on: false }
  if (busy === 'stop') action = { ...action, on: false }

  const showBar = (busy === 'setup' || busy === 'update') && progress
  const pct = Math.min(100, Math.max(0, progress?.overall ?? 0))
  const tone = busy === 'play' || update?.available ? 'wait' : status.setup_done ? 'on' : 'off'

  return (
    <div className="stage">
      <div className="stage-paper" />
      <img src={hero} alt="" className="stage-hero enter-hero" draggable="false" />
      <div className="stage-burn" />

      {/* title bar: drag it to move the window */}
      <header data-tauri-drag-region className="absolute inset-x-0 top-0 z-10 h-[60px] flex items-center justify-between pl-14 pr-4">
        <div className="flex items-center gap-7">
          <Emblem className="w-7 h-7 text-ink pointer-events-none" />
          <nav className="flex items-center gap-6">
            <NavLink onClick={() => setModal('accounts')}>Accounts</NavLink>
            <NavLink onClick={() => setModal('settings')}>Settings</NavLink>
          </nav>
        </div>
        <div className="flex items-center gap-1">
          <HideButton onClick={minimizeWindow} />
          <CloseButton onClick={closeWindow} label="Close (stops the realm)" />
        </div>
      </header>

      <main className="absolute left-14 top-[60px] bottom-0 w-[400px] flex flex-col">
        <section className="pt-[92px]">
          <p className="eyebrow enter" style={{ '--d': '60ms' }}>Realm 3.3.5a</p>
          <h1 className="wordmark text-[64px] mt-3 enter" style={{ '--d': '120ms' }}>Emberfall</h1>
          <span className="break mt-5 -ml-1 enter" style={{ '--d': '180ms' }} aria-hidden="true" />
          <div className="flex gap-2 mt-6 enter" style={{ '--d': '240ms' }}>
            <StatusChip on={status.mysql_running}>Database {status.mysql_running ? 'on' : 'off'}</StatusChip>
            <StatusChip on={status.server_running}>Realm {status.server_running ? 'online' : 'offline'}</StatusChip>
          </div>
        </section>

        <section className="mt-auto enter" style={{ '--d': '320ms' }} aria-live="polite">
          {showBar ? (
            <div className="mb-3">
              <div className="flex items-baseline gap-3 text-[13px]">
                <span className="font-semibold text-ink truncate">{progress.label}</span>
                <span className="font-display font-bold text-[15px] text-ink tabular-nums ml-auto">{pct.toFixed(0)}%</span>
              </div>
              <div className="-mx-[6px] mt-0.5"><Bar pct={pct} label={progress.label} /></div>
              <p className="text-[12px] text-ink-2 truncate -mt-1">
                {progress.total > 0 && progress.stage !== 'package' && progress.done > 1000 ? `${mb(progress.done)} of ${mb(progress.total)} MB · ` : ''}
                {progress.note}
              </p>
            </div>
          ) : (
            <div className="mb-4">
              <div className="flex items-center gap-2.5">
                <Indicator tone={tone} className="w-[18px] h-[18px]" />
                <span className="font-display font-bold text-[18px] text-ink">{busy === 'play' && phase ? phase.message : action.state}</span>
                {!busy && action.detail && <span className="text-[13px] text-ink-2">{action.detail}</span>}
              </div>
              {msg && <p className={`mt-2 text-[13px] font-semibold ${msg.tone === 'bad' ? 'text-bad' : 'text-ok'}`}>{msg.text}</p>}
            </div>
          )}
          <BigButton className="w-[300px]" disabled={!action.on} onClick={action.run}>
            {action.label}
          </BigButton>
        </section>

        <footer className="mt-7 mb-6 text-[11.5px] leading-[1.6] text-ink-2">
          <div>Server files {status.installed_version ? `v${status.installed_version}` : 'not installed'} · Launcher v{status.launcher_version}</div>
          <div className="truncate">Game folder: {status.client_dir || 'not set'}</div>
        </footer>
      </main>

      {/* first run: point to the game folder */}
      {!status.client_dir && (
        <Modal title="Game folder" width={430}>
          <p className="text-[14px] leading-relaxed text-center text-ink">
            Select your 3.3.5a game folder, the one with <b>Wow.exe</b> and <b>Data</b>.
          </p>
          {msg && <p className="text-[13px] font-semibold text-bad text-center mt-3">{msg.text}</p>}
          <div className="flex justify-center mt-5 mb-1">
            <Button className="w-[210px]" onClick={chooseClient}>Choose game folder</Button>
          </div>
        </Modal>
      )}

      {modal === 'settings' && (
        <Modal title="Settings" onClose={() => setModal(null)} width={560}>
          <div className="space-y-4">
            <Field label="Game folder">
              <div className="flex gap-2 items-center">
                <div className="wow-edit"><span className="truncate">{status.client_dir || 'not set'}</span></div>
                <Button className="w-[100px] shrink-0" onClick={chooseClient}>Change</Button>
              </div>
            </Field>
            <Field label="Server files folder">
              <div className="flex gap-2 items-center">
                <div className="wow-edit"><span className="truncate">{status.server_dir}</span></div>
                <Button className="w-[100px] shrink-0" onClick={chooseServer} disabled={!!busy}>Change</Button>
              </div>
            </Field>
            <div className="flex items-center gap-2 pt-2 border-t border-ink/15">
              <Button className="w-[170px] mt-2" onClick={recheck}>Check for updates</Button>
              <Button
                className="w-[150px] mt-2"
                disabled={!!busy}
                onClick={() => run('stop', async () => { await invoke('stop_realm'); await refresh(); say('ok', 'Realm stopped.') })}
              >
                Stop the realm
              </Button>
              <span className={`text-[12px] ml-auto mt-2 font-semibold ${update?.error ? 'text-bad' : 'text-ink-2'}`}>{update?.error ? update.error : update?.latest ? `Latest v${update.latest}` : ''}</span>
            </div>
          </div>
        </Modal>
      )}

      {modal === 'accounts' && <AccountsModal status={status} onClose={() => setModal(null)} />}
    </div>
  )
}

function AccountsModal({ status, onClose }) {
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
    <Modal title="Accounts" onClose={onClose} width={420}>
      <div className="tabs mb-4" role="tablist">
        {['create', 'delete'].map((t) => (
          <button key={t} role="tab" aria-selected={tab === t} onClick={() => { setTab(t); setOut(null) }} className="tab">
            {t === 'create' ? 'Create' : 'Delete'}
          </button>
        ))}
      </div>
      {!status.setup_done ? (
        <p className="text-[13px] text-ink-2 pb-2">Install the realm files first.</p>
      ) : (
        <form onSubmit={submit} className="flex flex-col gap-3">
          <Field label="Account name">
            <input className="wow-edit" autoComplete="off" maxLength={16} value={f.user} onChange={(e) => setF({ ...f, user: e.target.value })} />
          </Field>
          <Field label={tab === 'delete' ? 'Password (to confirm)' : 'Password'}>
            <input className="wow-edit" type="password" autoComplete="off" value={f.pass} onChange={(e) => setF({ ...f, pass: e.target.value })} />
          </Field>
          {tab === 'create' ? (
            <label className="flex items-center gap-2 text-[13px] font-medium cursor-pointer text-ink">
              <Check checked={f.gm} onChange={(e) => setF({ ...f, gm: e.target.checked })} />
              GM account (level 3)
            </label>
          ) : (
            <p className="text-[12.5px] text-ink-2">Accounts with characters cannot be deleted.</p>
          )}
          <div className="flex justify-center mt-1">
            <Button className="w-[190px]" disabled={working || !f.user || !f.pass}>
              {working ? 'Working ...' : tab === 'create' ? 'Create account' : 'Delete account'}
            </Button>
          </div>
          {out && <p className={`text-[13px] text-center font-semibold ${out.ok ? 'text-ok' : 'text-bad'}`}>{out.text}</p>}
        </form>
      )}
    </Modal>
  )
}
