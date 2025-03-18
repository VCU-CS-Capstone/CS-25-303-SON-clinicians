export interface UserSessionData {
  created: string;
  expires: string;
  session_key: string;
  login_id: string;
  user_id: number;
}
export class UserSession {
  created: Date;
  expires: Date;
  session_key: string;
  login_id: string;
  user_id: number;
  constructor(data: UserSessionData) {
    this.created = new Date(data.created);
    this.expires = new Date(data.expires);
    this.session_key = data.session_key;
    this.login_id = data.login_id;
    this.user_id = data.user_id;
  }

  hasExpired(): boolean {
    return this.expires < new Date();
  }
}
export interface User {
  id: number;
  first_name: string;
  last_name: string;
}

export interface LoginResponse {
  session: UserSessionData;
  user: User;
}
