GNUPGHOME=./gpghome
DOC_NAME=test-doc
dcore=../../target/debug/cli
USER_FINGERPRINT=2490D2F1CE64B972780203DA3A287C4593CDC4D2

clear
rm -rf $DOC_NAME

echo "create a document"
echo ""

GNUPGHOME=$GNUPGHOME $dcore document-create -u $USER_FINGERPRINT -d $DOC_NAME

echo "list all resources of the document"
echo ""

GNUPGHOME=$GNUPGHOME  $dcore resource-list-all -u $USER_FINGERPRINT -d $DOC_NAME

echo "Cat the content of a specific resource"
echo ""

GNUPGHOME=$GNUPGHOME  $dcore resource-cat -u $USER_FINGERPRINT -d $DOC_NAME -r config

echo "Mutate the content of a resource"
echo ""

GNUPGHOME=$GNUPGHOME  $dcore resource-set -u $USER_FINGERPRINT \
                                                      -d $DOC_NAME \
                                                      -r config \
                                                      -k ${USER_FINGERPRINT}.remote  \
                                                      -v https://github.com/alice


echo "Cat the mutated resource"
echo ""
GNUPGHOME=$GNUPGHOME  $dcore resource-cat -u $USER_FINGERPRINT -d $DOC_NAME -r config


### Add resource
GNUPGHOME=$GNUPGHOME  $dcore resource-add -u $USER_FINGERPRINT -d $DOC_NAME --resource-name test
GNUPGHOME=$GNUPGHOME  $dcore resource-list-all -u $USER_FINGERPRINT -d $DOC_NAME
GNUPGHOME=$GNUPGHOME  $dcore resource-cat -u $USER_FINGERPRINT -d $DOC_NAME --resource-name test

GNUPGHOME=$GNUPGHOME  $dcore resource-set -u $USER_FINGERPRINT \
                                                      -d $DOC_NAME \
                                                      -r test \
                                                      -k hello  \
                                                      -v dcore
GNUPGHOME=$GNUPGHOME  $dcore resource-cat -u $USER_FINGERPRINT -d $DOC_NAME --resource-name test
