<#
  Request: GET /store/order/{orderId}
  Summary: Find purchase order by ID
  Description: For valid response try integer IDs with value >= 1 and <= 10. Other values will generated exceptions
#>
param(
   <# ID of pet that needs to be fetched #>
   [Parameter(Mandatory=$True)]
   [String] $orderid
)

curl -X GET https://petstore.swagger.io/v2/store/order/$orderId?orderId=$orderid `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `

