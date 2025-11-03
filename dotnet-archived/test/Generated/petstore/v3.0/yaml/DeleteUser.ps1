<#
  Request: DELETE /user/{username}
  Summary: Delete user
  Description: This can only be done by the logged in user.
#>
param(
   <# The name that needs to be deleted #>
   [Parameter(Mandatory=$True)]
   [String] $username
)

curl -X DELETE /user/$username?username=$username `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `

