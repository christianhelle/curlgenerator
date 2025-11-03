#
# Request: GET /history
# Summary: User Activity
# Description: The User Activity endpoint returns data about a user's lifetime activity with Uber. The response will include pickup locations and times, dropoff locations and times, the distance of past requests, and information about which products were requested.<br><br>The history array in the response will have a maximum length based on the limit parameter. The response value count may exceed limit, therefore subsequent API requests may be necessary.
#

# Offset the list of returned results by this amount. Default is zero.
offset=""
# Number of items to retrieve. Default is 5, maximum is 100.
limit=""

curl -X GET "https://api.uber.com/v1/history?offset=${offset}&limit=${limit}" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
