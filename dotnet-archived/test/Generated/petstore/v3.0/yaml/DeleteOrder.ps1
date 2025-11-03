<#
  Request: DELETE /store/order/{orderId}
  Summary: Delete purchase order by ID
  Description: For valid response try integer IDs with value < 1000. Anything above 1000 or nonintegers will generate API errors
#>
param(
   <# ID of the order that needs to be deleted #>
   [Parameter(Mandatory=$True)]
   [String] $orderid
)

curl -X DELETE /store/order/$orderId?orderId=$orderid `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `

