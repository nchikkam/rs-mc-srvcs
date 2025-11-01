#!/bin/sh
function send_request() {
    echo -ne "- - - - - - - - - \nRequest: $1\nResponse ($2): "
    curl --header "Content-Type: application/json" --request POST \
         --data "$1" \
         "http://localhost:8080/random?format=$2"
    echo ""
}

send_request '{"distribution": "uniform", "parameters": {"start": -100, "end": 100}}' json
send_request '{"distribution": "uniform", "parameters": {"start": -100, "end": 100}}' cbor
send_request '{"distribution": "uniform", "parameters": {"start": -100, "end": 100}}' xml

<<SAMPLEOUTPUT
./test.sh 
-ne - - - - - - - - - 
Request: {"distribution": "uniform", "parameters": {"start": -100, "end": 100}}
Response (json): 
{"value":14.0}
-ne - - - - - - - - - 
Request: {"distribution": "uniform", "parameters": {"start": -100, "end": 100}}
Response (cbor): 
Warning: Binary output can mess up your terminal. Use "--output -" to tell 
Warning: curl to output it to your terminal anyway, or consider "--output 
Warning: <FILE>" to save to a file.

-ne - - - - - - - - - 
Request: {"distribution": "uniform", "parameters": {"start": -100, "end": 100}}
Response (xml): 
unsupported format xml
SAMPLEOUTPUT
