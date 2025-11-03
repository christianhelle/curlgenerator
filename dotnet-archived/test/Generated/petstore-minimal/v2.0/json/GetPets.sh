#
# Request: GET /pets
# Description: Returns all pets from the system that the user has access to
#

curl -X GET "http://petstore.swagger.io/api/pets" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
