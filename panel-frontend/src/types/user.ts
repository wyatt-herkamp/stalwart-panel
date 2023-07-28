export interface PanelUser {
  id: number
  name: string
  username: string
  email: string
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
