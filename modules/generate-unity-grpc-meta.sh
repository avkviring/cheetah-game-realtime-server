generated_scripts=$(find -regex '.*/Unity/.*\(Editor\|Runtime\)/GRPC.*.cs')
for f in $generated_scripts; do
    project_part=$(basename $(dirname $(dirname $f)))
    file_name=$(basename $f)

echo "fileFormatVersion: 2
guid: $(uuidgen --md5 -n @url -N Unity/$project_part/$file_name | tr -d '-')
MonoImporter:
externalObjects: {}
serializedVersion: 2
defaultReferences: []
executionOrder: 0
icon: {instanceID: 0}
userData: 
assetBundleName: 
assetBundleVariant: 
" > $f.meta

done
