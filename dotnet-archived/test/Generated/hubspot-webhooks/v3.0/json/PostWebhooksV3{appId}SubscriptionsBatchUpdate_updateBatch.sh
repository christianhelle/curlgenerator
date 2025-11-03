#
# Request: POST /webhooks/v3/{appId}/subscriptions/batch/update
#

# path parameter: appid
appid=""

curl -X POST "https://api.hubapi.com//webhooks/v3/$appId/subscriptions/batch/update" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
  -d '{
  "inputs": [
    {
      "id": 0,
      "active": false
    }
  ]
}'

