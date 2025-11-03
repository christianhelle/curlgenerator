#
# Request: PUT /webhooks/v3/{appId}/settings
#

# path parameter: appid
appid=""

curl -X PUT "https://api.hubapi.com//webhooks/v3/$appId/settings" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
  -d '{
  "targetUrl": "https://www.example.com/hubspot/target",
  "throttling": {
    "maxConcurrentRequests": 10,
    "period": "SECONDLY"
  }
}'

