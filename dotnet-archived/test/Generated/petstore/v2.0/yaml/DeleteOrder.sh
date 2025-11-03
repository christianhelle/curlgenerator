#
# Request: DELETE /store/order/{orderId}
# Summary: Delete purchase order by ID
# Description: For valid response try integer IDs with positive integer value. Negative or non-integer values will generate API errors
#

# ID of the order that needs to be deleted
orderid=""

curl -X DELETE "https://petstore.swagger.io/v2/store/order/$orderId" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
