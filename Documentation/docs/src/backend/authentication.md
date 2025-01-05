# Authentication

## Overview

Currently due to the early state of this project, Authentication is very basic.

Currently you will start by making a POST request to the
`/api/user/login/password​` endpoint. With a JSON body containing
`username` and `password`.

like so:

```json
{
  "username": "some_username",
  "password": "some_password"
}
```

This will return a JSON object with a user object containing the users information and a session object. Which contains a session_id field which you will use to authenticate future requests.

See [`/api/user/login/password`](https://cs-25-303.wyatt-herkamp.dev/scalar#tag/user/POST/api/user/login/password) for more information.

After that you will either need to set a cookie with the name of `session` and the value of the `{session_id}`. You may also pass the session id into the Authorization header as a `Session` scheme.

Like `Authorization: Session {session_id}`

## Future
- SSO Support Via SAML