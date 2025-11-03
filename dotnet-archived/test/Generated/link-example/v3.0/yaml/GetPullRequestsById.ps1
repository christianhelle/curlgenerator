<#
  Request: GET /2.0/repositories/{username}/{slug}/pullrequests/{pid}
#>
param(
   [Parameter(Mandatory=$True)]
   [String] $username,

   [Parameter(Mandatory=$True)]
   [String] $slug,

   [Parameter(Mandatory=$True)]
   [String] $pid
)

curl -X GET /2.0/repositories/$username/$slug/pullrequests/$pid?username=$username&slug=$slug&pid=$pid `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `

