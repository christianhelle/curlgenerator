#
# Request: DELETE /pets/{id}
# Description: deletes a single pet based on the ID supplied
#

# ID of pet to delete
id=""

curl -X DELETE "http://petstore.swagger.io/api/pets/$id" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
