#
# Request: GET /store/order/{orderId}
# Summary: Find purchase order by ID
# Description: For valid response try integer IDs with value >= 1 and <= 10. Other values will generated exceptions
#

# ID of pet that needs to be fetched
orderid=""

curl -X GET "https://petstore.swagger.io/v2/store/order/$orderId" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
