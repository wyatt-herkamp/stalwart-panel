export enum EmailType {
  Primary = 'Primary',
  Alias = 'Alias',
  Group = 'Group'
}
export interface Email {
  id: number
  account: number
  email_address: string
  email_type: EmailType
  created: Date
}
