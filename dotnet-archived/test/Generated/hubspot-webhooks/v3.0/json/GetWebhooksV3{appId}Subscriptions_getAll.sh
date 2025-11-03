#
# Request: GET /webhooks/v3/{appId}/subscriptions
#

# path parameter: appid
appid=""

curl -X GET "https://api.hubapi.com//webhooks/v3/$appId/subscriptions" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
