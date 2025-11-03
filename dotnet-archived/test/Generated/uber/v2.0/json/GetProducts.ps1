<#
  Request: GET /products
  Summary: Product Types
  Description: The Products endpoint returns information about the Uber products offered at a given location. The response includes the display name and other details about each product, and lists the products in the proper display order.
#>
param(
   <# Latitude component of location. #>
   [Parameter(Mandatory=$True)]
   [String] $latitude,

   <# Longitude component of location. #>
   [Parameter(Mandatory=$True)]
   [String] $longitude
)

curl -X GET https://api.uber.com/v1/products?latitude=$latitude&longitude=$longitude `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `

