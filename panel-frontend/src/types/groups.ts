export interface GroupPermissions {
  modify_accounts: boolean
  manage_system: boolean
}
export interface Group {
  id: number
  group_name: string
  group_permissions: GroupPermissions
  created: Date
}
