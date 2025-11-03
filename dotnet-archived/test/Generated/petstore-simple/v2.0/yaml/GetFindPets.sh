#
# Request: GET /pets
# Description: Returns all pets from the system that the user has access to
#

# tags to filter by
tags=""
# maximum number of results to return
limit=""

curl -X GET "http://petstore.swagger.io/api/pets?tags=${tags}&limit=${limit}" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
