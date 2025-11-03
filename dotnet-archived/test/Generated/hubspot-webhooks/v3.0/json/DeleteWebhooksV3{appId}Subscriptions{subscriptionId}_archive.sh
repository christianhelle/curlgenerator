#
# Request: DELETE /webhooks/v3/{appId}/subscriptions/{subscriptionId}
#

# path parameter: subscriptionid
subscriptionid=""
# path parameter: appid
appid=""

curl -X DELETE "https://api.hubapi.com//webhooks/v3/$appId/subscriptions/$subscriptionId" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
