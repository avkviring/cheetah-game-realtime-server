#!/bin/bash

kubectl create namespace system
helm -n system upgrade --install system System
