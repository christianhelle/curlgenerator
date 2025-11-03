<#
  Request: POST /pet/{petId}
  Summary: Updates a pet in the store with form data
#>
param(
   <# ID of pet that needs to be updated #>
   [Parameter(Mandatory=$True)]
   [String] $petid,

   <# Name of pet that needs to be updated #>
   [Parameter(Mandatory=$True)]
   [String] $name,

   <# Status of pet that needs to be updated #>
   [Parameter(Mandatory=$True)]
   [String] $status
)

curl -X POST /api/v3/pet/$petId?petId=$petid&name=$name&status=$status `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `

