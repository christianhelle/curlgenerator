<#
  Request: GET /me
  Summary: User Profile
  Description: The User Profile endpoint returns information about the Uber user that has authorized with the application.
#>

curl -X GET https://api.uber.com/v1/me `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `

