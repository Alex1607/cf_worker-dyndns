# DynDNS

Automatically Update Cloudflare DNS entries with a Cloudflare worker.  

## Usage
To use this worker you can call the URL of the worker with the following parameters: ``/?ipv4=x.x.x.x&apikey=XXXXX`` or `/?ipv6=::1&apikey=XXXXX`  
For now only IPv4 and IPv6 entries can be used and only one subdomain is supported per worker.
