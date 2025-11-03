<#
  Request: GET /webhooks/v3/{appId}/subscriptions
#>
param(
   [Parameter(Mandatory=$True)]
   [String] $appid
)

curl -X GET https://api.hubapi.com//webhooks/v3/$appId/subscriptions?appId=$appid `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `

