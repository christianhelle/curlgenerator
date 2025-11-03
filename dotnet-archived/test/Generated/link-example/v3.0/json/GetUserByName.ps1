<#
  Request: GET /2.0/users/{username}
#>
param(
   [Parameter(Mandatory=$True)]
   [String] $username
)

curl -X GET /2.0/users/$username?username=$username `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `

