<#
  Request: GET /2.0/repositories/{username}
#>
param(
   [Parameter(Mandatory=$True)]
   [String] $username
)

curl -X GET /2.0/repositories/$username?username=$username `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `

