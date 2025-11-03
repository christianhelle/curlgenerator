<#
  Request: GET /store/order/{orderId}
  Summary: Find purchase order by ID
  Description: For valid response try integer IDs with value <= 5 or > 10. Other values will generated exceptions
#>
param(
   <# ID of order that needs to be fetched #>
   [Parameter(Mandatory=$True)]
   [String] $orderid
)

curl -X GET /store/order/$orderId?orderId=$orderid `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `

