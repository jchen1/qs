# OAuth with GraphQL

1. User clicks "Login" button
2. Client makes `OAuthServiceURL` GraphQL query
3. Client redirects to the provided URL (or in an iframe?)
4. Service redirects to `/oauth/{service}/callback?code={}`
5. Client sends mutation `FinishOAuthServiceFlow`
6. After successful login, change route to `/`