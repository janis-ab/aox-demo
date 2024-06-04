#!/bin/sh

#
# This is just a simple startup script that allows us to run built
# service when it is available.
#
# Intended only for service demonstration purposes.
#

echo "Starting service"

fn_service=/service/service_demo_rel/target/release/service_demo
# fn_service=/service/service_demo/target/debug/service_demo

#
# The idea for main script is to loop here untill we have a service
# file available.
#
while [ ! -f "$fn_service" ]; do
    echo "Service not found at: $fn_service"
    sleep 5
done

cd /service/service_demo_rel

echo "Starting process at $fn_service"
$fn_service

echo "Demo process is exiting"
