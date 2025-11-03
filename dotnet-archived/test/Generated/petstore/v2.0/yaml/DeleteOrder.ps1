<#
  Request: DELETE /store/order/{orderId}
  Summary: Delete purchase order by ID
  Description: For valid response try integer IDs with positive integer value. Negative or non-integer values will generate API errors
#>
param(
   <# ID of the order that needs to be deleted #>
   [Parameter(Mandatory=$True)]
   [String] $orderid
)

curl -X DELETE https://petstore.swagger.io/v2/store/order/$orderId?orderId=$orderid `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `

