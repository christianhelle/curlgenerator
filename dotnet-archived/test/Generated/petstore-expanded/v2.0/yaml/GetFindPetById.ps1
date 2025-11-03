<#
  Request: GET /pets/{id}
  Description: Returns a user based on a single ID, if the user does not have access to the pet
#>
param(
   <# ID of pet to fetch #>
   [Parameter(Mandatory=$True)]
   [String] $id
)

curl -X GET http://petstore.swagger.io/api/pets/$id?id=$id `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `

