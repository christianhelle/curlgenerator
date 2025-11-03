<#
  Request: POST /streams
  Description: subscribes a client to receive out-of-band data
#>
param(
   <# the location where data will be sent.  Must be network accessible
by the source server
 #>
   [Parameter(Mandatory=$True)]
   [String] $callbackurl
)

curl -X POST /streams?callbackUrl=$callbackurl `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `

