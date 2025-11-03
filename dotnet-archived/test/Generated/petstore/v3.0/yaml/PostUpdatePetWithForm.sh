#
# Request: POST /pet/{petId}
# Summary: Updates a pet in the store with form data
#

# ID of pet that needs to be updated
petid=""
# Name of pet that needs to be updated
name=""
# Status of pet that needs to be updated
status=""

curl -X POST "/pet/$petId?name=${name}&status=${status}" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
