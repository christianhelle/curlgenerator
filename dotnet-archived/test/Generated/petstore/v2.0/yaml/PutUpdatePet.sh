#
# Request: PUT /pet
# Summary: Update an existing pet
#

curl -X PUT "https://petstore.swagger.io/v2/pet" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
  -d '{
  "id": 0,
  "category": {
    "id": 0,
    "name": "string"
  },
  "name": "doggie",
  "photoUrls": [
    "string"
  ],
  "tags": [
    {
      "id": 0,
      "name": "string"
    }
  ],
  "status": "string"
}'

