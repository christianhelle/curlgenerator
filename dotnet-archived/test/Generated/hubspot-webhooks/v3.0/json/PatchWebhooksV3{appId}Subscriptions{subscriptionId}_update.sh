#
# Request: PATCH /webhooks/v3/{appId}/subscriptions/{subscriptionId}
#

# path parameter: subscriptionid
subscriptionid=""
# path parameter: appid
appid=""

curl -X PATCH "https://api.hubapi.com//webhooks/v3/$appId/subscriptions/$subscriptionId" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
  -d '{
  "active": true
}'

