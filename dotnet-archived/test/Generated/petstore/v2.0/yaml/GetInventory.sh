#
# Request: GET /store/inventory
# Summary: Returns pet inventories by status
# Description: Returns a map of status codes to quantities
#

curl -X GET "https://petstore.swagger.io/v2/store/inventory" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
