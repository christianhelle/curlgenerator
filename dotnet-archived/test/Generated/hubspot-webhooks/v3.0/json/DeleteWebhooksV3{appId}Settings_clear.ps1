<#
  Request: DELETE /webhooks/v3/{appId}/settings
#>
param(
   [Parameter(Mandatory=$True)]
   [String] $appid
)

curl -X DELETE https://api.hubapi.com//webhooks/v3/$appId/settings?appId=$appid `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `

