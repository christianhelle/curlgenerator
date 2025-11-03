<#
  Request: GET /2.0/repositories/{username}/{slug}/pullrequests
#>
param(
   [Parameter(Mandatory=$True)]
   [String] $username,

   [Parameter(Mandatory=$True)]
   [String] $slug,

   [Parameter(Mandatory=$True)]
   [String] $state
)

curl -X GET /2.0/repositories/$username/$slug/pullrequests?username=$username&slug=$slug&state=$state `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `

