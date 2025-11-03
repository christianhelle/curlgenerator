#
# Request: PUT /user/{username}
# Summary: Updated user
# Description: This can only be done by the logged in user.
#

# name that need to be updated
username=""

curl -X PUT "https://petstore.swagger.io/v2/user/$username" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
  -d '{
  "id": 0,
  "username": "string",
  "firstName": "string",
  "lastName": "string",
  "email": "string",
  "password": "string",
  "phone": "string",
  "userStatus": 0
}'

