<#
  Request: GET /pets
  Description: Returns all pets from the system that the user has access to
#>
param(
   <# tags to filter by #>
   [Parameter(Mandatory=$True)]
   [String] $tags,

   <# maximum number of results to return #>
   [Parameter(Mandatory=$True)]
   [String] $limit
)

curl -X GET http://petstore.swagger.io/api/pets?tags=$tags&limit=$limit `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `

