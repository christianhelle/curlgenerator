<#
  Request: POST /2.0/repositories/{username}/{slug}/pullrequests/{pid}/merge
#>
param(
   [Parameter(Mandatory=$True)]
   [String] $username,

   [Parameter(Mandatory=$True)]
   [String] $slug,

   [Parameter(Mandatory=$True)]
   [String] $pid
)

curl -X POST /2.0/repositories/$username/$slug/pullrequests/$pid/merge?username=$username&slug=$slug&pid=$pid `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `

