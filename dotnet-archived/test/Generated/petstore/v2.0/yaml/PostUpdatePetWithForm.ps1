<#
  Request: POST /pet/{petId}
  Summary: Updates a pet in the store with form data
#>
param(
   <# ID of pet that needs to be updated #>
   [Parameter(Mandatory=$True)]
   [String] $petid
)

curl -X POST https://petstore.swagger.io/v2/pet/$petId?petId=$petid `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `

