declare const __APP_VERSION__: string
declare const __COMMIT_ID__: string
declare const __IS_TAURI__: boolean

declare module 'vue-virtual-scroller'
declare interface Navigator {
  standalone?: boolean
}

type ToolTipParams = {
  data: {
    value: number
    name: number
  }
  seriesName: string
  color: string
}
