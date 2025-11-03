<#
  Request: POST /store/order
  Summary: Place an order for a pet
  Description: Place a new order in the store
#>

curl -X POST /store/order `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `
  -d '{
  "id": 10,
  "petId": 198772,
  "quantity": 7,
  "shipDate": "2025-09-25T10.30.58Z",
  "status": "approved",
  "complete": false
}'

