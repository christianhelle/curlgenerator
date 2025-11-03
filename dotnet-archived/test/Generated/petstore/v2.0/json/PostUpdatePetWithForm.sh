#
# Request: POST /pet/{petId}
# Summary: Updates a pet in the store with form data
#

# ID of pet that needs to be updated
petid=""
name=""
status=""

curl -X POST "https://petstore.swagger.io/v2/pet/$petId" \
  -H "Accept: application/json" \
  -H "Content-Type: application/x-www-form-urlencoded" \
-F "name=${name}" \
-F "status=${status}" \

