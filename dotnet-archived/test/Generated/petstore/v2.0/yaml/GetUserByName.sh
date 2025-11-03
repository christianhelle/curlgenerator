#
# Request: GET /user/{username}
# Summary: Get user by user name
#

# The name that needs to be fetched. Use user1 for testing. 
username=""

curl -X GET "https://petstore.swagger.io/v2/user/$username" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
