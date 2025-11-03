#
# Request: POST /user
# Summary: Create user
# Description: This can only be done by the logged in user.
#

curl -X POST "/user" \
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

