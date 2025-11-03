<#
  Request: GET /pet/findByStatus
  Summary: Finds Pets by status
  Description: Multiple status values can be provided with comma separated strings
#>
param(
   <# Status values that need to be considered for filter #>
   [Parameter(Mandatory=$True)]
   [String] $status
)

curl -X GET /api/v3/pet/findByStatus?status=$status `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `

