#
# Request: GET /estimates/price
# Summary: Price Estimates
# Description: The Price Estimates endpoint returns an estimated price range for each product offered at a given location. The price estimate is provided as a formatted string with the full price range and the localized currency symbol.<br><br>The response also includes low and high estimates, and the [ISO 4217](http://en.wikipedia.org/wiki/ISO_4217) currency code for situations requiring currency conversion. When surge is active for a particular product, its surge_multiplier will be greater than 1, but the price estimate already factors in this multiplier.
#

# Latitude component of start location.
start_latitude=""
# Longitude component of start location.
start_longitude=""
# Latitude component of end location.
end_latitude=""
# Longitude component of end location.
end_longitude=""

curl -X GET "https://api.uber.com/v1/estimates/price?start_latitude=${start_latitude}&start_longitude=${start_longitude}&end_latitude=${end_latitude}&end_longitude=${end_longitude}" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
