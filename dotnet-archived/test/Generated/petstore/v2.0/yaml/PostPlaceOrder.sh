#
# Request: POST /store/order
# Summary: Place an order for a pet
#

curl -X POST "https://petstore.swagger.io/v2/store/order" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
  -d '{
  "id": 0,
  "petId": 0,
  "quantity": 0,
  "shipDate": "2025-09-25T10.30.48Z",
  "status": "string",
  "complete": false
}'

