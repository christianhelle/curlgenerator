#
# Request: POST /webhooks/v3/{appId}/subscriptions
#

# path parameter: appid
appid=""

curl -X POST "https://api.hubapi.com//webhooks/v3/$appId/subscriptions" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
  -d '{
  "active": true,
  "eventType": "contact.propertyChange",
  "propertyName": "email"
}'

