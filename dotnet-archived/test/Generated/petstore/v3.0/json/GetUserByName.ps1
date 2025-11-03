<#
  Request: GET /user/{username}
  Summary: Get user by user name
#>
param(
   <# The name that needs to be fetched. Use user1 for testing.  #>
   [Parameter(Mandatory=$True)]
   [String] $username
)

curl -X GET /api/v3/user/$username?username=$username `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `

