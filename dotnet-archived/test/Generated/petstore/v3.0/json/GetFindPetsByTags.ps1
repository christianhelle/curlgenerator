<#
  Request: GET /pet/findByTags
  Summary: Finds Pets by tags
  Description: Multiple tags can be provided with comma separated strings. Use tag1, tag2, tag3 for testing.
#>
param(
   <# Tags to filter by #>
   [Parameter(Mandatory=$True)]
   [String] $tags
)

curl -X GET /api/v3/pet/findByTags?tags=$tags `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `

