<#
  Request: PATCH /webhooks/v3/{appId}/subscriptions/{subscriptionId}
#>
param(
   [Parameter(Mandatory=$True)]
   [String] $subscriptionid,

   [Parameter(Mandatory=$True)]
   [String] $appid
)

curl -X PATCH https://api.hubapi.com//webhooks/v3/$appId/subscriptions/$subscriptionId?subscriptionId=$subscriptionid&appId=$appid `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `
  -d '{
  "active": true
}'

