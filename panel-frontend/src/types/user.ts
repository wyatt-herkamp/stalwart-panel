export interface PanelUser {
  id: number
  name: string
  username: string
  email: string
  group_permissions: GroupPermissions
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

export interface GroupPermissions {
  modify_accounts: boolean
  manage_system: boolean
}
