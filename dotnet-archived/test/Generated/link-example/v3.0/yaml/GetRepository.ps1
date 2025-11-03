<#
  Request: GET /2.0/repositories/{username}/{slug}
#>
param(
   [Parameter(Mandatory=$True)]
   [String] $username,

   [Parameter(Mandatory=$True)]
   [String] $slug
)

curl -X GET /2.0/repositories/$username/$slug?username=$username&slug=$slug `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `

