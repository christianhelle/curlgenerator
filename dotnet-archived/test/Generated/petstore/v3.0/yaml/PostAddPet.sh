#
# Request: POST /pet
# Summary: Add a new pet to the store
# Description: Add a new pet to the store
#

curl -X POST "/pet" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
  -d '{
  "id": 10,
  "name": "doggie",
  "category": {
    "id": 1,
    "name": "Dogs"
  },
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

