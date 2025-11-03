#
# Request: PUT /user/{username}
# Summary: Update user
# Description: This can only be done by the logged in user.
#

# name that need to be deleted
username=""

curl -X PUT "/api/v3/user/$username" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
  -d '{
  "id": 10,
  "username": "theUser",
  "firstName": "John",
  "lastName": "James",
  "email": "john@email.com",
  "password": "12345",
  "phone": "12345",
  "userStatus": 1
}'

