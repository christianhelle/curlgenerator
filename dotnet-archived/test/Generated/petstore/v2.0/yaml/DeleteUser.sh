#
# Request: DELETE /user/{username}
# Summary: Delete user
# Description: This can only be done by the logged in user.
#

# The name that needs to be deleted
username=""

curl -X DELETE "https://petstore.swagger.io/v2/user/$username" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
