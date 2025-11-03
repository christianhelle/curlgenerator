#
# Request: GET /pet/{petId}
# Summary: Find pet by ID
# Description: Returns a single pet
#

# ID of pet to return
petid=""

curl -X GET "/pet/$petId" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
