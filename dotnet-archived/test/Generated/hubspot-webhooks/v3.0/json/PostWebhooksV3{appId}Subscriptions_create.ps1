<#
  Request: POST /webhooks/v3/{appId}/subscriptions
#>
param(
   [Parameter(Mandatory=$True)]
   [String] $appid
)

curl -X POST https://api.hubapi.com//webhooks/v3/$appId/subscriptions?appId=$appid `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `
  -d '{
  "active": true,
  "eventType": "contact.propertyChange",
  "propertyName": "email"
}'

