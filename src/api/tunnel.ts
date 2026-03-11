import { invoke } from '@tauri-apps/api/core'
import { ref } from 'vue'

export const isTauri = typeof __IS_TAURI__ !== 'undefined' && __IS_TAURI__
const isMobileDevice = /Android|iPhone|iPad|iPod/i.test(navigator.userAgent)
export const isDesktopTauri = isTauri && !isMobileDevice

export interface RustTunnelConfig {
  id: string
  name: string
  backend_uuid: string
  tool: string
  args: string[]
  local_port: number
  auto_start: boolean
}

export interface RustTunnelStatus {
  id: string
  running: boolean
  pid: number | null
  error: string | null
  logs: string[]
}

export const tunnelStatuses = ref<Map<string, RustTunnelStatus>>(new Map())

export async function getTunnels(): Promise<RustTunnelConfig[]> {
  if (!isTauri) return []
  return await invoke<RustTunnelConfig[]>('get_tunnels')
}

export async function getTunnelStatuses(): Promise<RustTunnelStatus[]> {
  if (!isTauri) return []
  const statuses = await invoke<RustTunnelStatus[]>('get_tunnel_statuses')
  statuses.forEach((s) => tunnelStatuses.value.set(s.id, s))
  return statuses
}

export async function saveTunnel(config: RustTunnelConfig): Promise<void> {
  if (!isTauri) return
  await invoke('save_tunnel', { config })
}

export async function removeTunnel(id: string): Promise<void> {
  if (!isTauri) return
  await invoke('remove_tunnel', { id })
  tunnelStatuses.value.delete(id)
}

export async function startTunnel(id: string): Promise<RustTunnelStatus> {
  if (!isTauri) throw new Error('Not in Tauri')
  const status = await invoke<RustTunnelStatus>('start_tunnel', { id })
  tunnelStatuses.value.set(id, status)
  return status
}

export async function stopTunnel(id: string): Promise<RustTunnelStatus> {
  if (!isTauri) throw new Error('Not in Tauri')
  const status = await invoke<RustTunnelStatus>('stop_tunnel', { id })
  tunnelStatuses.value.set(id, status)
  return status
}

export async function addDefenderExclusion(): Promise<string> {
  if (!isTauri) throw new Error('Not in Tauri')
  return await invoke<string>('add_defender_exclusion')
}

// Convert frontend TunnelConfig to Rust format
export function toRustTunnelConfig(
  backendUuid: string,
  tunnel: { tool: string; args: string; localPort: number; autoStart: boolean },
): RustTunnelConfig {
  return {
    id: backendUuid,
    name: '',
    backend_uuid: backendUuid,
    tool: tunnel.tool,
    args: tunnel.args.split(/\s+/).filter((a) => a.length > 0),
    local_port: tunnel.localPort,
    auto_start: tunnel.autoStart,
  }
}
