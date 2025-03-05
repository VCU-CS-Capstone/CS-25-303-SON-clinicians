import { LoginResponse } from "./types/user.ts";
import { User } from "./types/user.ts";
import os from "node:os";

const API_URL = "https://cs-25-303.wyatt-herkamp.dev";
// User Agent Header Value Please let it start it with CS25-30x
// X being the group number
function userAgent() {
  return `CS25-303 Example Client / Deno / ${os.type}`;
}
export class CS25Client {
  apiURL: string;
  sessionKey: string | undefined;

  constructor(apiURL: string = API_URL) {
    if (apiURL.endsWith("/")) {
      apiURL = apiURL.slice(0, -1);
    }
    this.apiURL = apiURL;
  }
  appendEndpoint(endpoint: string) {
    if (endpoint.startsWith("/")) {
      return `${this.apiURL}${endpoint}`;
    } else {
      return `${this.apiURL}/${endpoint}`;
    }
  }

  async get(endpoint: string) {
    const url = this.appendEndpoint(endpoint);
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers
    const headers: Record<string, string> = {
      "User-Agent": userAgent(),
      Accept: "application/json",
    };
    if (this.sessionKey) {
      // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Authorization
      headers["Authorization"] = `Session ${this.sessionKey}`;
    }
    const response = await fetch(url, {
      method: "GET",
      headers: headers,
      // Just have this to prevent cors errors. I think. This is something I dont fully understand
      credentials: "include",
    });
    //console.trace("Get Response", response);
    if (!response.ok) {
      throw new Error(`Failed to fetch ${endpoint}, Error: ${response.status}`);
    }
    return response;
  }
  // Same as get but will return the JSON body of the response
  async getJson(endpoint: string) {
    const response = await this.get(endpoint);
    return await response.json();
  }
  // HTTP Post request with JSON body
  // deno-lint-ignore no-explicit-any
  async postJson(endpoint: string, data: any) {
    const url = this.appendEndpoint(endpoint);
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers
    const headers: Record<string, string> = {
      "User-Agent": userAgent(),
      // If we are sending JSON we need to tell the server so we are saying the content type of our body is JSON
      "Content-Type": "application/json",
    };
    if (this.sessionKey) {
      // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Authorization
      headers["Authorization"] = `Session ${this.sessionKey}`;
    }

    const response = await fetch(url, {
      method: "POST",
      headers: headers,
      body: JSON.stringify(data),
      // Just have this to prevent cors errors. I think. This is something I dont fully understand
      credentials: "include",
    });
    //console.trace("Post Response", response);
    if (!response.ok) {
      throw new Error(`Failed to post ${endpoint}, Error: ${response.status}`);
    }
    return response;
  }
  // Same as postJson but will return the JSON body of the response
  async postJsonJsonBody(endpoint: string, data: any) {
    const response = await this.postJson(endpoint, data);
    return await response.json();
  }
  // Once the SessionKey is set it will be passed into all requests under the Authorization header
  setSessionKey(sessionKey: string) {
    this.sessionKey = sessionKey;
  }
  /// Logs a user in. Responds with the user object.
  // Throws an error if the login fails.
  async login(username: string, password: string): Promise<User> {
    const response = await this.postJson("/api/auth/login/password", {
      username,
      password,
    });
    // Should be status code 200
    if (!response.ok) {
      throw new Error(`Failed to login, Error: ${response.status}`);
    }
    const body = (await response.json()) as LoginResponse;
    this.setSessionKey(body.session.session_key);
    return body.user;
  }
}
