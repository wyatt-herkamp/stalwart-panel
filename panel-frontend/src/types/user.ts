import type { Email } from '@/types/emails'
import type { GroupPermissions } from '@/types/groups'
import type { DropDownOption } from '@/components/form/FormTypes'

export interface PanelUser {
  id: number
  name: string
  username: string
  email: string
  group_permissions: GroupPermissions
}
export interface FullUser {
  id: number
  name: string
  username: string
  description: string
  requires_password_change: boolean
  quota: number
  account_type: AccountType
  active: boolean
  backup_email?: string
  created: Date
  group_id: number
  group_name: string
  group_permissions: GroupPermissions
  emails?: Email[]
}
export function pickName(name: string) {
  return name.split(' ')[0]
}
export interface Session {
  user_id: number
  session_id: string
  expires: Date
  created: Date
}
export interface LoginResponse {
  panel_user: PanelUser
  session: Session
}

export enum AccountType {
  'Individual' = 'Individual',
  'Group' = 'Group'
}
export namespace AccountType {
  export function options(): Array<DropDownOption<AccountType>> {
    return [
      { name: 'Individual', value: AccountType.Individual },
      { name: 'Group', value: AccountType.Group }
    ]
  }
}
export interface AccountSimple {
  id: number
  name: string
  username: string
  description: string
  account_type: AccountType
  primary_email?: string
  active: boolean
}
