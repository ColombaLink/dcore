#!/bin/bash

PS4="\n\n\033[1;33m>>>\033[0m"; set -x
export GNUPGHOME=./gpghome
DOC_NAME=test-doc
dcore=../../target/debug/cli
#dcore=./dcore

function wait {
    set +x
    read -p "Press a key to continue" -n 1
    clear
    PS4="\n\n\033[1;33m>>>\033[0m"; set -x
}

rm -rf ./gpghome
rm -rf $DOC_NAME
mkdir ./gpghome

clear
$dcore

wait

$dcore identity-create --name "Alice" --email "info@colomba.link"

wait

$dcore identity-list-all

wait

USER_FINGERPRINT=$( $dcore identity-list-all | grep -o 'Fingerprint.*' | awk '{print $2}')

$dcore document-create -u $USER_FINGERPRINT -d $DOC_NAME --device-name notebook1

wait

$dcore resource-list-all -u $USER_FINGERPRINT -d $DOC_NAME
wait
$dcore resource-cat -u $USER_FINGERPRINT -d $DOC_NAME --resource-name config
wait
$dcore resource-set -u $USER_FINGERPRINT \
                                                      -d $DOC_NAME \
                                                      -r config \
                                                      -k ${USER_FINGERPRINT}.remote  \
                                                      -v git@github.com:fuubi/gpgtest.git

wait
$dcore resource-cat -u $USER_FINGERPRINT -d $DOC_NAME --resource-name config

wait

#sync
$dcore document-sync -u $USER_FINGERPRINT -d $DOC_NAME

wait

# new resource
$dcore resource-add -u $USER_FINGERPRINT -d $DOC_NAME --resource-name test
wait
$dcore resource-list-all -u $USER_FINGERPRINT -d $DOC_NAME
wait
$dcore resource-cat -u $USER_FINGERPRINT -d $DOC_NAME --resource-name test

wait
$dcore resource-set -u $USER_FINGERPRINT \
                                                      -d $DOC_NAME \
                                                      -r test \
                                                      -k hello  \
                                                      -v dcore
wait
$dcore resource-cat -u $USER_FINGERPRINT -d $DOC_NAME --resource-name test
