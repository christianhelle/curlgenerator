#
# Request: GET /pets/{id}
# Description: Returns a user based on a single ID, if the user does not have access to the pet
#

# ID of pet to fetch
id=""

curl -X GET "http://petstore.swagger.io/api/pets/$id" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
