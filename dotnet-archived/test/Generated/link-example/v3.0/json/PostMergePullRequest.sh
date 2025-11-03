#
# Request: POST /2.0/repositories/{username}/{slug}/pullrequests/{pid}/merge
#

# path parameter: username
username=""
# path parameter: slug
slug=""
# path parameter: pid
pid=""

curl -X POST "/2.0/repositories/$username/$slug/pullrequests/$pid/merge" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
