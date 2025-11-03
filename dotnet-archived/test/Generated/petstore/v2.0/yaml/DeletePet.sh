#
# Request: DELETE /pet/{petId}
# Summary: Deletes a pet
#

# header parameter: api_key
api_key=""
# Pet id to delete
petid=""

curl -X DELETE "https://petstore.swagger.io/v2/pet/$petId" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
