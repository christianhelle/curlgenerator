#
# Request: GET /2.0/repositories/{username}/{slug}/pullrequests/{pid}
#

# path parameter: username
username=""
# path parameter: slug
slug=""
# path parameter: pid
pid=""

curl -X GET "/2.0/repositories/$username/$slug/pullrequests/$pid" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
