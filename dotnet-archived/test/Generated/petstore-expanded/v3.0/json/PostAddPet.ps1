<#
  Request: POST /pets
  Description: Creates a new pet in the store. Duplicates are allowed
#>

curl -X POST http://petstore.swagger.io/api/pets `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `
  -d '{
  "name": "string",
  "tag": "string"
}'

