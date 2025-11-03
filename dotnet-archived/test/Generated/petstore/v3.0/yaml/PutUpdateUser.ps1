<#
  Request: PUT /user/{username}
  Summary: Update user
  Description: This can only be done by the logged in user.
#>
param(
   <# name that need to be deleted #>
   [Parameter(Mandatory=$True)]
   [String] $username
)

curl -X PUT /user/$username?username=$username `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `
  -d '{
  "id": 10,
  "username": "theUser",
  "firstName": "John",
  "lastName": "James",
  "email": "john@email.com",
  "password": "12345",
  "phone": "12345",
  "userStatus": 1
}'

