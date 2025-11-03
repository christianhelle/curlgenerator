#
# Request: DELETE /pet/{petId}
# Summary: Deletes a pet
#

# 
api_key=""
# Pet id to delete
petid=""

curl -X DELETE "/api/v3/pet/$petId" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
