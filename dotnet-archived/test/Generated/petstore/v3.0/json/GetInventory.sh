#
# Request: GET /store/inventory
# Summary: Returns pet inventories by status
# Description: Returns a map of status codes to quantities
#

curl -X GET "/api/v3/store/inventory" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
