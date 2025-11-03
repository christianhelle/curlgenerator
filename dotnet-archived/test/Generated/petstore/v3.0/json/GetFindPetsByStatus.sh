#
# Request: GET /pet/findByStatus
# Summary: Finds Pets by status
# Description: Multiple status values can be provided with comma separated strings
#

# Status values that need to be considered for filter
status=""

curl -X GET "/api/v3/pet/findByStatus?status=${status}" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
