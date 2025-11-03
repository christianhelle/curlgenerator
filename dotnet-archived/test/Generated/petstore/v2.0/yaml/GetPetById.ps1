<#
  Request: GET /pet/{petId}
  Summary: Find pet by ID
  Description: Returns a single pet
#>
param(
   <# ID of pet to return #>
   [Parameter(Mandatory=$True)]
   [String] $petid
)

curl -X GET https://petstore.swagger.io/v2/pet/$petId?petId=$petid `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `

