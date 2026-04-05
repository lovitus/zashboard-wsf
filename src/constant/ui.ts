import {
  ArrowsRightLeftIcon,
  Cog6ToothIcon,
  CubeTransparentIcon,
  DocumentTextIcon,
  GlobeAltIcon,
  ServerIcon,
  SwatchIcon,
} from '@heroicons/vue/24/outline'
import { ROUTE_NAME, SETTINGS_MENU_KEY } from './index'

export type BuiltinRouteMeta = {
  titleKey: string
  subtitle: string
  icon: object
}

export const BUILTIN_ROUTE_META: Record<string, BuiltinRouteMeta> = {
  [ROUTE_NAME.overview]: {
    titleKey: ROUTE_NAME.overview,
    subtitle: 'Realtime charts, health, and traffic snapshots.',
    icon: CubeTransparentIcon,
  },
  [ROUTE_NAME.proxies]: {
    titleKey: ROUTE_NAME.proxies,
    subtitle: 'Switch groups, test latency, and review providers.',
    icon: GlobeAltIcon,
  },
  [ROUTE_NAME.connections]: {
    titleKey: ROUTE_NAME.connections,
    subtitle: 'Inspect live sessions and close noisy traffic quickly.',
    icon: ArrowsRightLeftIcon,
  },
  [ROUTE_NAME.rules]: {
    titleKey: ROUTE_NAME.rules,
    subtitle: 'Audit rule matches and refresh rule providers.',
    icon: SwatchIcon,
  },
  [ROUTE_NAME.logs]: {
    titleKey: ROUTE_NAME.logs,
    subtitle: 'Filter runtime logs and export incidents when needed.',
    icon: DocumentTextIcon,
  },
  [ROUTE_NAME.settings]: {
    titleKey: ROUTE_NAME.settings,
    subtitle: 'Tune appearance, connection handling, and backend behavior.',
    icon: Cog6ToothIcon,
  },
  [ROUTE_NAME.setup]: {
    titleKey: ROUTE_NAME.setup,
    subtitle: 'Manage backends, native services, and built-in control access.',
    icon: ServerIcon,
  },
}

export type SettingsSectionMeta = {
  titleKey: string
  subtitle: string
}

export const SETTINGS_SECTION_META: Record<SETTINGS_MENU_KEY, SettingsSectionMeta> = {
  [SETTINGS_MENU_KEY.general]: {
    titleKey: 'general',
    subtitle: 'Theme, interaction, and global built-in UI behavior.',
  },
  [SETTINGS_MENU_KEY.overview]: {
    titleKey: 'overviewSettings',
    subtitle: 'Overview cards, startup checks, and sidebar summary behavior.',
  },
  [SETTINGS_MENU_KEY.backend]: {
    titleKey: 'backendSettings',
    subtitle: 'Ports, actions, updates, and backend runtime controls.',
  },
  [SETTINGS_MENU_KEY.proxies]: {
    titleKey: 'proxySettings',
    subtitle: 'Latency testing, proxy cards, and icon presentation.',
  },
  [SETTINGS_MENU_KEY.connections]: {
    titleKey: 'connectionSettings',
    subtitle: 'Connection layout, table options, and source labeling.',
  },
}
