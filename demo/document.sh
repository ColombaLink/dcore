GNUPGHOME=./gpghome
DOC_NAME=test-doc
dcore=../target/debug/dcore
USER_FINGERPRINT=456D60DC8418C8C9613028E768D84598B97DD116

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
