#
# Request: DELETE /webhooks/v3/{appId}/settings
#

# path parameter: appid
appid=""

curl -X DELETE "https://api.hubapi.com//webhooks/v3/$appId/settings" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
