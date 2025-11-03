#
# Request: GET /store/order/{orderId}
# Summary: Find purchase order by ID
# Description: For valid response try integer IDs with value <= 5 or > 10. Other values will generate exceptions.
#

# ID of order that needs to be fetched
orderid=""

curl -X GET "/api/v3/store/order/$orderId" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
