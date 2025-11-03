<#
  Request: POST /user
  Summary: Create user
  Description: This can only be done by the logged in user.
#>

curl -X POST https://petstore.swagger.io/v2/user `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `
  -d '{
  "id": 0,
  "username": "string",
  "firstName": "string",
  "lastName": "string",
  "email": "string",
  "password": "string",
  "phone": "string",
  "userStatus": 0
}'

