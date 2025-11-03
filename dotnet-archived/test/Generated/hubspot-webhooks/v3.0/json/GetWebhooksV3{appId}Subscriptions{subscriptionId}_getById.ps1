<#
  Request: GET /webhooks/v3/{appId}/subscriptions/{subscriptionId}
#>
param(
   [Parameter(Mandatory=$True)]
   [String] $subscriptionid,

   [Parameter(Mandatory=$True)]
   [String] $appid
)

curl -X GET https://api.hubapi.com//webhooks/v3/$appId/subscriptions/$subscriptionId?subscriptionId=$subscriptionid&appId=$appid `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `

