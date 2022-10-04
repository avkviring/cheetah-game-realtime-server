#!/bin/bash
set -e 
helm cheetah-config-creator ../../Assets/Editor/Cheetah/Production/ charts/Config/templates
helmwave up --build --kubedog
