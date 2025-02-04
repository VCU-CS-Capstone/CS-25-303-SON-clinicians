export interface UserSession {
  created: string;
  expires: string;
  session_key: string;
  login_id: string;
}
export interface User {
  id: number;
  first_name: string;
  last_name: string;
}
