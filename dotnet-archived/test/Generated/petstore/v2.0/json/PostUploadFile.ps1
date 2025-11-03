<#
  Request: POST /pet/{petId}/uploadImage
  Summary: uploads an image
#>
param(
   <# ID of pet to update #>
   [Parameter(Mandatory=$True)]
   [String] $petid
)

curl -X POST https://petstore.swagger.io/v2/pet/$petId/uploadImage?petId=$petid `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `

