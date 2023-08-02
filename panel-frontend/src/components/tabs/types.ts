import { Ref } from 'vue'

export interface TabValue {
  name: string
  id: string
  disabled?: boolean
  icon?: string
  updateContent?: boolean
}

export interface TabEvent {
  name: string
  id: string
  disabled: boolean
  updateContent: boolean
}
export interface TabData {
  currentTab: Ref<string>
  update: (tab: TabEvent) => void
}
