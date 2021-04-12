#!/bin/bash 
if [[ -n $1 ]]
then
    rm $1*.log $1*.json $1*.csv -f
else
    rm *.log *.json *.csv /tmp/.tmp* /tmp/*.o /tmp/emphermal_record.* /tmp/*.o.tmp -rf
fi