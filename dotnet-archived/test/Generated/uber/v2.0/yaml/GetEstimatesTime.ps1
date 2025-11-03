<#
  Request: GET /estimates/time
  Summary: Time Estimates
  Description: The Time Estimates endpoint returns ETAs for all products offered at a given location, with the responses expressed as integers in seconds. We recommend that this endpoint be called every minute to provide the most accurate, up-to-date ETAs.
#>
param(
   <# Latitude component of start location. #>
   [Parameter(Mandatory=$True)]
   [String] $start_latitude,

   <# Longitude component of start location. #>
   [Parameter(Mandatory=$True)]
   [String] $start_longitude,

   <# Unique customer identifier to be used for experience customization. #>
   [Parameter(Mandatory=$True)]
   [String] $customer_uuid,

   <# Unique identifier representing a specific product for a given latitude & longitude. #>
   [Parameter(Mandatory=$True)]
   [String] $product_id
)

curl -X GET https://api.uber.com/v1/estimates/time?start_latitude=$start_latitude&start_longitude=$start_longitude&customer_uuid=$customer_uuid&product_id=$product_id `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `

