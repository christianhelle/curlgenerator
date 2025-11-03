<#
  Request: DELETE /pet/{petId}
  Summary: Deletes a pet
#>
param(
   <# Pet id to delete #>
   [Parameter(Mandatory=$True)]
   [String] $petid
)

curl -X DELETE /api/v3/pet/$petId?petId=$petid `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `

