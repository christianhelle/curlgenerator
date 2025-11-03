<#
  Request: POST /pet/{petId}/uploadImage
  Summary: uploads an image
#>
param(
   <# ID of pet to update #>
   [Parameter(Mandatory=$True)]
   [String] $petid,

   <# Additional Metadata #>
   [Parameter(Mandatory=$True)]
   [String] $additionalmetadata
)

curl -X POST /pet/$petId/uploadImage?petId=$petid&additionalMetadata=$additionalmetadata `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `

