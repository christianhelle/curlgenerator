#
# Request: POST /user/createWithList
# Summary: Creates list of users with given input array
# Description: Creates list of users with given input array
#

curl -X POST "/api/v3/user/createWithList" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
  -d '[
  {
    "id": 10,
    "username": "theUser",
    "firstName": "John",
    "lastName": "James",
    "email": "john@email.com",
    "password": "12345",
    "phone": "12345",
    "userStatus": 1
  }
]'

