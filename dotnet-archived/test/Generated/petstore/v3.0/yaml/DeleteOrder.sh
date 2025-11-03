#
# Request: DELETE /store/order/{orderId}
# Summary: Delete purchase order by ID
# Description: For valid response try integer IDs with value < 1000. Anything above 1000 or nonintegers will generate API errors
#

# ID of the order that needs to be deleted
orderid=""

curl -X DELETE "/store/order/$orderId" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
