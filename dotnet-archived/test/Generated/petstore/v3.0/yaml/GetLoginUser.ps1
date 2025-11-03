<#
  Request: GET /user/login
  Summary: Logs user into the system
#>
param(
   <# The user name for login #>
   [Parameter(Mandatory=$True)]
   [String] $username,

   <# The password for login in clear text #>
   [Parameter(Mandatory=$True)]
   [String] $password
)

curl -X GET /user/login?username=$username&password=$password `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `

