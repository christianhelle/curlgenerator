#
# Request: GET /user/login
# Summary: Logs user into the system
#

# The user name for login
username=""
# The password for login in clear text
password=""

curl -X GET "/user/login?username=${username}&password=${password}" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
