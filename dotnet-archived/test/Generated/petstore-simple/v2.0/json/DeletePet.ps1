<#
  Request: DELETE /pets/{id}
  Description: deletes a single pet based on the ID supplied
#>
param(
   <# ID of pet to delete #>
   [Parameter(Mandatory=$True)]
   [String] $id
)

curl -X DELETE http://petstore.swagger.io/api/pets/$id?id=$id `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `

