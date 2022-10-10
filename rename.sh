#!/bin/bash

cd migrations

number=0
for a in *.up.*
do
  NAME=$(printf "%02d_%s\n" $(( ++number )) $(echo $a | sed 's/[0-9_]//g'))
  mv $a $NAME
done

number=0
for a in *.down.*
do
  NAME=$(printf "%02d_%s\n" $(( ++number )) $(echo $a | sed 's/[0-9_]//g'))
  mv $a $NAME
done
