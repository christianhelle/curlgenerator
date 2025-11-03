<#
  Request: POST /user/createWithList
  Summary: Creates list of users with given input array
#>

curl -X POST https://petstore.swagger.io/v2/user/createWithList `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `
  -d '[
  {
    "id": 0,
    "username": "string",
    "firstName": "string",
    "lastName": "string",
    "email": "string",
    "password": "string",
    "phone": "string",
    "userStatus": 0
  }
]'

